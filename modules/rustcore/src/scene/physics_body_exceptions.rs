//! `PhysicsBody2D`/`PhysicsBody3D` collision-exception lists (state only).
//!
//! `add_collision_exception_with`, `remove_collision_exception_with` and
//! `get_collision_exceptions` maintain the set of bodies this body ignores. The
//! engine forwards the calls to the physics server and answers the getter from
//! the server's list; this port keeps the same list locally (`add` refuses
//! duplicates, `remove` reports whether the body was present).
//!
//! Moving bodies, `move_and_collide` and the server calls are not ported.

use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct CollisionExceptions {
    bodies: Vec<Rid>,
}

impl CollisionExceptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, body: Rid) -> bool {
        if self.bodies.contains(&body) {
            return false;
        }
        self.bodies.push(body);
        true
    }

    pub fn remove(&mut self, body: Rid) -> bool {
        match self.bodies.iter().position(|other| *other == body) {
            Some(index) => {
                self.bodies.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self) {
        self.bodies.clear();
    }

    pub fn has(&self, body: Rid) -> bool {
        self.bodies.contains(&body)
    }

    pub fn get(&self) -> &[Rid] {
        &self.bodies
    }

    pub fn count(&self) -> usize {
        self.bodies.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove_and_list() {
        let mut e = CollisionExceptions::new();
        assert_eq!(e.count(), 0);
        assert!(e.get().is_empty());
        assert!(!e.has(Rid::from_u64(1)));

        assert!(e.add(Rid::from_u64(1)));
        assert!(!e.add(Rid::from_u64(1)), "no duplicates");
        assert!(e.add(Rid::from_u64(2)));
        assert_eq!(e.count(), 2);
        assert_eq!(e.get(), &[Rid::from_u64(1), Rid::from_u64(2)]);
        assert!(e.has(Rid::from_u64(2)));

        assert!(e.remove(Rid::from_u64(1)));
        assert!(!e.remove(Rid::from_u64(1)));
        assert_eq!(e.get(), &[Rid::from_u64(2)]);

        e.clear();
        assert_eq!(e.count(), 0);
    }
}
