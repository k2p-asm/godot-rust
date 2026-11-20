//! `core/os/os.*` (environment subset): process environment access.
//!
//! The engine exposes the platform environment through `OS`: `get_environment`,
//! `set_environment`, `has_environment`, `unset_environment` and
//! `get_environment_list`. This port maps them to `std::env`; failures are
//! reported as `false` instead of `ERR_FAIL` logs.
//!
//! The rest of `OS` (platform, memory, threads, dialogs) lives elsewhere or is
//! not ported.

/// `OS::get_environment`.
pub fn get_environment(variable: &str) -> Option<String> {
    std::env::var(variable).ok()
}

/// `OS::set_environment`.
pub fn set_environment(variable: &str, value: &str) -> bool {
    if variable.is_empty() {
        return false;
    }
    std::env::set_var(variable, value);
    true
}

/// `OS::has_environment`.
pub fn has_environment(variable: &str) -> bool {
    std::env::var_os(variable).is_some()
}

/// `OS::unset_environment`; false when the variable was not set.
pub fn unset_environment(variable: &str) -> bool {
    if std::env::var_os(variable).is_none() {
        return false;
    }
    std::env::remove_var(variable);
    true
}

/// `OS::get_cmdline_args`: the process arguments, including the executable.
pub fn get_cmdline_args() -> Vec<String> {
    std::env::args().collect()
}

/// `OS::get_cmdline_user_args`: the arguments after a lone `--`.
pub fn get_cmdline_user_args() -> Vec<String> {
    let mut found = false;
    let mut user_args = Vec::new();
    for arg in std::env::args().skip(1) {
        if found {
            user_args.push(arg);
        } else if arg == "--" {
            found = true;
        }
    }
    user_args
}

/// `OS::get_environment_list`: every variable name, sorted (the engine returns
/// the platform's order; sorting makes it deterministic).
pub fn get_environment_list() -> Vec<String> {
    let mut names: Vec<String> = std::env::vars_os()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_name() -> String {
        format!(
            "GODOT_RUST_ENV_TEST_{}_{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    }

    #[test]
    fn set_get_unset_round_trip() {
        let name = unique_name();
        assert!(!has_environment(&name));
        assert_eq!(get_environment(&name), None);
        assert!(!unset_environment(&name), "nothing to remove");

        assert!(set_environment(&name, "value 1"));
        assert!(has_environment(&name));
        assert_eq!(get_environment(&name).as_deref(), Some("value 1"));

        // Overwriting works.
        assert!(set_environment(&name, "value 2"));
        assert_eq!(get_environment(&name).as_deref(), Some("value 2"));

        // The name shows up in the sorted list.
        let list = get_environment_list();
        assert!(list.contains(&name));
        let mut sorted = list.clone();
        sorted.sort();
        assert_eq!(list, sorted);

        assert!(unset_environment(&name));
        assert!(!has_environment(&name));
        assert_eq!(get_environment(&name), None);
    }

    #[test]
    fn invalid_names() {
        assert!(!set_environment("", "x"));
        // An unset variable is absent but querying it is safe.
        assert!(!has_environment(""));
        assert_eq!(get_environment("").as_deref(), None);
        assert!(!unset_environment(""));
    }
}
