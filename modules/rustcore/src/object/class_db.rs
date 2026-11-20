//! `ClassDB`-lite: class registration, inheritance lookup and method metadata,
//! keyed by `StringName` and bound to live objects by `ObjectId`.
//!
//! What is modelled: the shape the engine's `ClassDB` relies on for dispatch --
//! a class hierarchy, per-class method names with arity, and a mapping from a
//! live object to its class. Lookups walk the parent chain, as
//! `ClassDB::class_has_method` does with `p_no_inheritance = false`.
//!
//! NOT modelled: property lists, signals, virtual methods, `_bind_methods`
//! argument marshalling and the actual function pointers (`MethodBind`). Those
//! need `Object`, `Variant` dispatch and the binding layer, so this module is
//! **non-differential**: `class_db.cpp` cannot be linked into the harness.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::object::ObjectId;
use crate::string::string_name::StringName;
use crate::variant::Variant;

/// Everything a bound method receives: the object and its arguments.
pub struct CallCtx {
    pub object: ObjectId,
    pub args: Vec<Variant>,
}

impl CallCtx {
    pub fn arg(&self, i: usize) -> Option<&Variant> {
        self.args.get(i)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallError {
    UnknownObject,
    UnknownMethod,
    WrongArgCount { expected: i32, got: usize },
    BadArguments,
}

pub type MethodFn = fn(&CallCtx) -> Result<Variant, CallError>;

#[derive(Clone, Copy)]
struct BoundMethod {
    argc: i32,
    f: MethodFn,
}

#[derive(Clone)]
struct ClassInfo {
    parent: Option<StringName>,
    /// method name -> argument count
    methods: HashMap<StringName, i32>,
    bindings: HashMap<StringName, BoundMethod>,
    signals: Vec<StringName>,
    properties: Vec<StringName>,
}

#[derive(Default)]
struct Registry {
    classes: HashMap<StringName, ClassInfo>,
    /// ObjectId -> class of that instance.
    instances: HashMap<ObjectId, StringName>,
    /// (source, signal, target, method)
    connections: Vec<(ObjectId, StringName, ObjectId, StringName)>,
    /// (class, property) -> declared type
    property_types: HashMap<(StringName, StringName), crate::variant::VariantType>,
    property_meta: HashMap<(StringName, StringName), PropertyMeta>,
    /// (object, property) -> value.
    values: HashMap<(ObjectId, StringName), Variant>,
}

fn registry() -> &'static Mutex<Registry> {
    static REG: OnceLock<Mutex<Registry>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(Registry::default()))
}

/// Registers a class, optionally inheriting from `parent`.
/// Re-registering an existing class keeps its methods (idempotent).
pub fn register_class(name: &str, parent: Option<&str>) {
    let mut reg = registry().lock().unwrap();
    let key = StringName::new(name);
    reg.classes
        .entry(key)
        .or_insert_with(|| ClassInfo {
            parent: parent.map(StringName::new),
            methods: HashMap::new(),
            bindings: HashMap::new(),
            properties: Vec::new(),
            signals: Vec::new(),
        });
}

/// Registers a method on a class; the class must exist first.
pub fn register_method(class: &str, method: &str, argc: i32) {
    let mut reg = registry().lock().unwrap();
    let key = StringName::new(class);
    if let Some(info) = reg.classes.get_mut(&key) {
        info.methods.insert(StringName::new(method), argc);
    }
}

/// Binds a live object to its class (`Object::_class_name`).
pub fn bind_instance(id: ObjectId, class: &str) {
    let mut reg = registry().lock().unwrap();
    reg.instances.insert(id, StringName::new(class));
    // Binding also registers the hierarchy root of that class if unknown.
    drop(reg);
}

/// Unbinds an object (called when it is freed).
pub fn unbind_instance(id: ObjectId) {
    registry().lock().unwrap().instances.remove(&id);
}

pub fn class_exists(class: &str) -> bool {
    registry().lock().unwrap().classes.contains_key(&StringName::new(class))
}

/// True when `class` is `ancestor` or derives from it.
pub fn is_parent_class(class: &str, ancestor: &str) -> bool {
    let reg = registry().lock().unwrap();
    let target = StringName::new(ancestor);
    let mut current = Some(StringName::new(class));
    let mut guard = 0;

    while let Some(name) = current {
        if name == target {
            return true;
        }
        guard += 1;
        if guard > 64 {
            break; // Defensive: malformed hierarchy.
        }
        current = reg.classes.get(&name).and_then(|i| i.parent);
    }

    false
}

/// `ClassDB::class_has_method(class, method, no_inheritance = false)`.
pub fn class_has_method(class: &str, method: &str) -> bool {
    method_argc(class, method).is_some()
}

/// Argument count of a method, walking the parent chain. `None` if unknown.
pub fn method_argc(class: &str, method: &str) -> Option<i32> {
    let reg = registry().lock().unwrap();
    let want = StringName::new(method);
    let mut current = Some(StringName::new(class));
    let mut guard = 0;

    while let Some(name) = current {
        if let Some(info) = reg.classes.get(&name) {
            if let Some(argc) = info.methods.get(&want) {
                return Some(*argc);
            }
            current = info.parent;
        } else {
            return None;
        }
        guard += 1;
        if guard > 64 {
            return None;
        }
    }

    None
}

/// Class of a live object, if it was bound.
pub fn object_class(id: ObjectId) -> Option<StringName> {
    registry().lock().unwrap().instances.get(&id).copied()
}

/// Dispatch check: does this object have this method, and with how many args?
pub fn object_has_method(id: ObjectId, method: &str) -> Option<i32> {
    let class = object_class(id)?;
    method_argc(&class.as_string(), method)
}

/// Registers a callable implementation on a class.
pub fn bind_method(class: &str, method: &str, argc: i32, f: MethodFn) {
    let mut reg = registry().lock().unwrap();
    let key = StringName::new(class);
    if let Some(info) = reg.classes.get_mut(&key) {
        info.methods.insert(StringName::new(method), argc);
        info.bindings.insert(StringName::new(method), BoundMethod { argc, f });
    }
}

/// Resolves a method on a class, walking parents: returns the binding and the
/// declaring class.
fn resolve(class: &StringName, method: &StringName) -> Option<BoundMethod> {
    let reg = registry().lock().unwrap();
    let mut current = Some(*class);
    let mut guard = 0;

    while let Some(name) = current {
        if let Some(info) = reg.classes.get(&name) {
            if let Some(b) = info.bindings.get(method) {
                return Some(*b);
            }
            current = info.parent;
        } else {
            return None;
        }
        guard += 1;
        if guard > 64 {
            return None;
        }
    }

    None
}

/// Calls a method on a live object (`Object::call`), checking arity first.
pub fn call(id: ObjectId, method: &str, args: Vec<Variant>) -> Result<Variant, CallError> {
    let class = object_class(id).ok_or(CallError::UnknownObject)?;
    let m = StringName::new(method);
    let bound = resolve(&class, &m).ok_or(CallError::UnknownMethod)?;

    if bound.argc >= 0 && args.len() != bound.argc as usize {
        return Err(CallError::WrongArgCount { expected: bound.argc, got: args.len() });
    }

    let ctx = CallCtx { object: id, args };
    (bound.f)(&ctx)
}
/// Registers a property name on a class (`ADD_PROPERTY` without the hints).
pub fn register_property(class: &str, property: &str) {
    let mut reg = registry().lock().unwrap();
    let key = StringName::new(class);
    if let Some(info) = reg.classes.get_mut(&key) {
        let p = StringName::new(property);
        if !info.properties.contains(&p) {
            info.properties.push(p);
        }
    }
}

/// `get_property_list`: own properties then inherited ones, parents last.
pub fn property_list(id: ObjectId) -> Vec<StringName> {
    let Some(class) = object_class(id) else {
        return Vec::new();
    };
    let reg = registry().lock().unwrap();
    let mut out = Vec::new();
    let mut current = Some(class);
    let mut guard = 0;

    while let Some(name) = current {
        if let Some(info) = reg.classes.get(&name) {
            out.extend(info.properties.iter().copied());
            current = info.parent;
        } else {
            break;
        }
        guard += 1;
        if guard > 64 {
            break;
        }
    }

    out
}

/// True when the class of `id` declares `property` (including inherited ones).
pub fn has_property(id: ObjectId, property: &str) -> bool {
    let want = StringName::new(property);
    property_list(id).contains(&want)
}

/// `Object::get(name)`; `None` when the property is not registered or unset.
pub fn property_get(id: ObjectId, property: &str) -> Option<Variant> {
    if !has_property(id, property) {
        return None;
    }
    let reg = registry().lock().unwrap();
    reg.values.get(&(id, StringName::new(property))).cloned()
}

/// `Object::set(name, value)`; false when the property is not registered.
pub fn property_set(id: ObjectId, property: &str, value: Variant) -> bool {
    if !has_property(id, property) {
        return false;
    }
    let mut reg = registry().lock().unwrap();
    reg.values.insert((id, StringName::new(property)), value);
    true
}

/// Drops every stored value for an object (called when it is freed).
pub fn clear_property_values(id: ObjectId) {
    let mut reg = registry().lock().unwrap();
    reg.values.retain(|(oid, _), _| *oid != id);
}

/// Registers a signal on a class (`ADD_SIGNAL`, names only).
pub fn register_signal(class: &str, signal: &str) {
    let mut reg = registry().lock().unwrap();
    let key = StringName::new(class);
    if let Some(info) = reg.classes.get_mut(&key) {
        let s = StringName::new(signal);
        if !info.signals.contains(&s) {
            info.signals.push(s);
        }
    }
}

/// Signals declared by the class of `id`, including inherited ones.
pub fn signal_list(id: ObjectId) -> Vec<StringName> {
    let Some(class) = object_class(id) else {
        return Vec::new();
    };
    let reg = registry().lock().unwrap();
    let mut out = Vec::new();
    let mut current = Some(class);
    let mut guard = 0;

    while let Some(name) = current {
        if let Some(info) = reg.classes.get(&name) {
            out.extend(info.signals.iter().copied());
            current = info.parent;
        } else {
            break;
        }
        guard += 1;
        if guard > 64 {
            break;
        }
    }

    out
}

pub fn has_signal(id: ObjectId, signal: &str) -> bool {
    signal_list(id).contains(&StringName::new(signal))
}

/// `Object::connect`; false for unknown signals or duplicate connections.
pub fn connect_signal(source: ObjectId, signal: &str, target: ObjectId, method: &str) -> bool {
    if !has_signal(source, signal) || object_class(target).is_none() {
        return false;
    }
    let mut reg = registry().lock().unwrap();
    let key = (source, StringName::new(signal), target, StringName::new(method));
    if reg.connections.contains(&key) {
        return false;
    }
    reg.connections.push(key);
    true
}

pub fn disconnect_signal(source: ObjectId, signal: &str, target: ObjectId, method: &str) -> bool {
    let mut reg = registry().lock().unwrap();
    let key = (source, StringName::new(signal), target, StringName::new(method));
    if let Some(pos) = reg.connections.iter().position(|c| *c == key) {
        reg.connections.remove(pos);
        true
    } else {
        false
    }
}

/// `emit_signal`: returns the `(target, method)` pairs to call, in connection
/// order. Dead targets are skipped. The caller dispatches them through `call`.
pub fn emit_signal(source: ObjectId, signal: &str) -> Vec<(ObjectId, StringName)> {
    if !has_signal(source, signal) {
        return Vec::new();
    }
    let reg = registry().lock().unwrap();
    let want = StringName::new(signal);
    reg.connections
        .iter()
        .filter(|(s, sig, t, _)| *s == source && *sig == want && reg.instances.contains_key(t))
        .map(|(_, _, t, m)| (*t, *m))
        .collect()
}

/// Drops connections where `id` is source or target (called when it is freed).
pub fn clear_signal_connections(id: ObjectId) {
    let mut reg = registry().lock().unwrap();
    reg.connections.retain(|(s, _, t, _)| *s != id && *t != id);
}

/// Registers a property name and its type on a class (`ADD_PROPERTY` plus the
/// `PROPERTY_HINT_NONE` default; hints/usages are not modelled yet).
pub fn register_typed_property(class: &str, property: &str, vtype: crate::variant::VariantType) {
    register_property(class, property);
    let mut reg = registry().lock().unwrap();
    reg.property_types.insert((StringName::new(class), StringName::new(property)), vtype);
}

/// Declared type of a property, walking the class hierarchy.
pub fn property_type(id: ObjectId, property: &str) -> Option<crate::variant::VariantType> {
    if !has_property(id, property) {
        return None;
    }
    let class = object_class(id)?;
    let reg = registry().lock().unwrap();
    let mut current = Some(class);
    let key = StringName::new(property);
    let mut guard = 0;

    while let Some(name) = current {
        if let Some(t) = reg.property_types.get(&(name, key)) {
            return Some(*t);
        }
        current = reg.classes.get(&name).and_then(|i| i.parent);
        guard += 1;
        if guard > 64 {
            break;
        }
    }

    None
}

/// Subset of `PropertyHint` (the engine has ~40 values; these are the ones the
/// port needs so far).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyHint {
    None,
    Range,
    Enum,
    Flags,
}

/// Subset of `PropertyUsageFlags`, as bit flags.
pub mod usage {
    pub const STORAGE: u32 = 1 << 1;
    pub const EDITOR: u32 = 1 << 2;
    pub const READ_ONLY: u32 = 1 << 6;
    /// Default the engine assigns to most exported properties.
    pub const DEFAULT: u32 = STORAGE | EDITOR;
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertyMeta {
    pub vtype: crate::variant::VariantType,
    pub hint: PropertyHint,
    pub hint_string: String,
    pub usage: u32,
}

/// Registers a property with its full metadata (`ADD_PROPERTY`).
pub fn register_property_ex(class: &str, property: &str, meta: PropertyMeta) {
    register_property(class, property);
    let mut reg = registry().lock().unwrap();
    let key = (StringName::new(class), StringName::new(property));
    reg.property_types.insert(key.clone(), meta.vtype);
    reg.property_meta.insert(key, meta);
}

/// Full metadata of a property, walking the class hierarchy.
pub fn property_meta(id: ObjectId, property: &str) -> Option<PropertyMeta> {
    if !has_property(id, property) {
        return None;
    }
    let class = object_class(id)?;
    let key = StringName::new(property);
    let reg = registry().lock().unwrap();
    let mut current = Some(class);
    let mut guard = 0;

    while let Some(name) = current {
        if let Some(m) = reg.property_meta.get(&(name, key)) {
            return Some(m.clone());
        }
        current = reg.classes.get(&name).and_then(|i| i.parent);
        guard += 1;
        if guard > 64 {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;
    use crate::object::Object;

    fn setup() {
        register_class("Object", None);
        register_class("Node", Some("Object"));
        register_class("Node3D", Some("Node"));
        register_method("Object", "free", 0);
        register_method("Object", "get_class", 0);
        register_method("Node", "add_child", 1);
        register_method("Node3D", "rotate_y", 1);
    }

    #[test]
    fn hierarchy_lookup() {
        setup();
        assert!(class_exists("Node3D"));
        assert!(!class_exists("Nonexistent"));

        assert!(is_parent_class("Node3D", "Node"));
        assert!(is_parent_class("Node3D", "Object"));
        assert!(is_parent_class("Node3D", "Node3D"));
        assert!(!is_parent_class("Node", "Node3D"));
        assert!(!is_parent_class("Node", "Node2D"));
    }

    #[test]
    fn methods_inherit_from_parents() {
        setup();
        // Own method.
        assert_eq!(method_argc("Node3D", "rotate_y"), Some(1));
        // Inherited twice.
        assert_eq!(method_argc("Node3D", "add_child"), Some(1));
        assert_eq!(method_argc("Node3D", "free"), Some(0));
        // Not present.
        assert_eq!(method_argc("Node3D", "not_a_method"), None);
        assert!(!class_has_method("Node", "rotate_y"));
    }

    #[test]
    fn instances_dispatch_by_id() {
        setup();
        let node = Object::new();
        bind_instance(node.id(), "Node3D");

        assert_eq!(object_class(node.id()).unwrap().as_string(), "Node3D");
        assert_eq!(object_has_method(node.id(), "rotate_y"), Some(1));
        assert_eq!(object_has_method(node.id(), "add_child"), Some(1));
        assert_eq!(object_has_method(node.id(), "nope"), None);

        let id = node.id();
        unbind_instance(id);
        assert!(object_class(id).is_none());
        assert_eq!(object_has_method(id, "rotate_y"), None);
    }

    #[test]
    fn unbound_object_has_no_class() {
        let obj = Object::new();
        assert!(object_class(obj.id()).is_none());
        assert_eq!(object_has_method(obj.id(), "free"), None);
    }

    #[test]
    fn dispatch_dispatches_and_checks_arity() {
        fn add(ctx: &CallCtx) -> Result<Variant, CallError> {
            match (ctx.arg(0), ctx.arg(1)) {
                (Some(Variant::Int(a)), Some(Variant::Int(b))) => Ok(Variant::Int(a + b)),
                _ => Err(CallError::BadArguments),
            }
        }
        fn whoami(ctx: &CallCtx) -> Result<Variant, CallError> {
            Ok(Variant::Object(ctx.object))
        }

        setup();
        bind_method("Node", "add_ints", 2, add);
        bind_method("Object", "whoami", 0, whoami);

        let node = Object::new();
        bind_instance(node.id(), "Node3D");

        // Inherited from Node.
        let r = call(node.id(), "add_ints", vec![Variant::Int(2), Variant::Int(3)]).unwrap();
        assert_eq!(r, Variant::Int(5));

        // Inherited twice, from Object.
        assert_eq!(call(node.id(), "whoami", vec![]).unwrap(), Variant::Object(node.id()));

        // Arity is validated before the call.
        assert_eq!(
            call(node.id(), "add_ints", vec![Variant::Int(1)]),
            Err(CallError::WrongArgCount { expected: 2, got: 1 })
        );

        // Wrong payload type surfaces as BadArguments.
        assert_eq!(
            call(node.id(), "add_ints", vec![Variant::Bool(true), Variant::Int(1)]),
            Err(CallError::BadArguments)
        );

        // Unknown method and unknown object.
        assert_eq!(call(node.id(), "nope", vec![]), Err(CallError::UnknownMethod));
        let unbound = Object::new();
        assert_eq!(call(unbound.id(), "add_ints", vec![]), Err(CallError::UnknownObject));
    }

    #[test]
    fn properties_inherit_and_store_values() {
        setup();
        register_property("Object", "name");
        register_property("Node", "owner");
        register_property("Node3D", "position");
        // Re-registering must not duplicate.
        register_property("Node3D", "position");

        let node = Object::new();
        bind_instance(node.id(), "Node3D");

        // Own property first, then inherited ones.
        let list: Vec<String> = property_list(node.id()).iter().map(|p| p.as_string()).collect();
        assert_eq!(list, vec!["position", "owner", "name"]);

        assert!(has_property(node.id(), "owner"));
        assert!(!has_property(node.id(), "not_a_property"));

        // Unset property reads as None.
        assert_eq!(property_get(node.id(), "position"), None);

        assert!(property_set(node.id(), "position", Variant::Vector3(Vector3::new(1.0, 2.0, 3.0))));
        assert_eq!(
            property_get(node.id(), "position"),
            Some(Variant::Vector3(Vector3::new(1.0, 2.0, 3.0)))
        );

        // Unknown properties cannot be written.
        assert!(!property_set(node.id(), "nope", Variant::Nil));

        // Freeing the object clears its values.
        clear_property_values(node.id());
        assert_eq!(property_get(node.id(), "position"), None);
    }

    #[test]
    fn signals_connect_emit_and_disconnect() {
        register_class("Node2", None);
        register_method("Node2", "on_died", 0);
        register_method("Node2", "on_hit", 0);
        register_signal("Node2", "died");
        register_signal("Node2", "hit");

        let src = Object::new();
        let a = Object::new();
        let b = Object::new();
        bind_instance(src.id(), "Node2");
        bind_instance(a.id(), "Node2");
        bind_instance(b.id(), "Node2");

        assert!(has_signal(src.id(), "died"));
        assert!(!has_signal(src.id(), "nope"));

        // Two targets on the same signal, plus a duplicate that must be refused.
        assert!(connect_signal(src.id(), "died", a.id(), "on_died"));
        assert!(connect_signal(src.id(), "died", b.id(), "on_died"));
        assert!(!connect_signal(src.id(), "died", a.id(), "on_died"));

        let delivered = emit_signal(src.id(), "died");
        assert_eq!(delivered.len(), 2);
        assert!(delivered.iter().any(|(t, _)| *t == a.id()));

        // Disconnect one target.
        assert!(disconnect_signal(src.id(), "died", a.id(), "on_died"));
        assert_eq!(emit_signal(src.id(), "died").len(), 1);
        assert!(!disconnect_signal(src.id(), "died", a.id(), "on_died"));

        // Unknown signal emits nothing.
        assert!(emit_signal(src.id(), "unknown").is_empty());

        // Freeing a target removes it from deliveries.
        let b_id = b.id();
        unbind_instance(b_id);
        assert!(emit_signal(src.id(), "died").is_empty());

        // Freeing the source clears everything.
        let src_id = src.id();
        clear_signal_connections(src_id);
        assert!(emit_signal(src_id, "died").is_empty());
    }

    #[test]
    fn property_types_inherit() {
        use crate::variant::VariantType;

        register_class("PObj", None);
        register_class("PNode", Some("PObj"));
        register_typed_property("PObj", "name", VariantType::String);
        register_typed_property("PNode", "position", VariantType::Vector3);

        let o = Object::new();
        bind_instance(o.id(), "PNode");

        assert_eq!(property_type(o.id(), "position"), Some(VariantType::Vector3));
        // Inherited from PObj.
        assert_eq!(property_type(o.id(), "name"), Some(VariantType::String));
        // Declared but untyped, and undeclared.
        register_property("PNode", "raw");
        assert_eq!(property_type(o.id(), "raw"), None);
        assert_eq!(property_type(o.id(), "missing"), None);
    }

    #[test]
    fn property_meta_carries_hint_and_usage() {
        use crate::variant::VariantType;

        register_class("MObj", None);
        register_property_ex(
            "MObj",
            "speed",
            PropertyMeta {
                vtype: VariantType::Float,
                hint: PropertyHint::Range,
                hint_string: "0,100,0.5".to_string(),
                usage: usage::DEFAULT,
            },
        );
        register_property_ex(
            "MObj",
            "state",
            PropertyMeta {
                vtype: VariantType::Int,
                hint: PropertyHint::Enum,
                hint_string: "idle,running".to_string(),
                usage: usage::STORAGE | usage::READ_ONLY,
            },
        );

        let o = Object::new();
        bind_instance(o.id(), "MObj");

        let speed = property_meta(o.id(), "speed").expect("speed has metadata");
        assert_eq!(speed.hint, PropertyHint::Range);
        assert_eq!(speed.hint_string, "0,100,0.5");
        assert_eq!(speed.usage & usage::EDITOR, usage::EDITOR);
        assert_eq!(speed.vtype, VariantType::Float);

        let state = property_meta(o.id(), "state").unwrap();
        assert_eq!(state.usage & usage::READ_ONLY, usage::READ_ONLY);
        assert_eq!(state.usage & usage::EDITOR, 0);

        assert!(property_meta(o.id(), "missing").is_none());
    }

    #[test]
    fn registration_is_idempotent() {
        setup();
        register_class("Node3D", Some("Node"));
        register_method("Node3D", "rotate_y", 1);
        assert_eq!(method_argc("Node3D", "rotate_y"), Some(1));
    }
}
