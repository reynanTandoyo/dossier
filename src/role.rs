//! Whether a session is a worker's or a planner's. A worker session is marked by the environment
//! variable DOSSIER_ROLE=worker (D56); every other session is a planner's.

/// The environment variable that marks a worker session.
pub const VARIABLE: &str = "DOSSIER_ROLE";

/// The role of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A session that plans and decides. This is the role of every session that is not marked.
    Planner,
    /// A session that works on one claimed task.
    Worker,
}

/// Worker when `value`, trimmed and in lower case, is `worker`; otherwise Planner.
pub fn of(value: Option<&str>) -> Role {
    match value.map(|value| value.trim().to_lowercase()) {
        Some(value) if value == "worker" => Role::Worker,
        _ => Role::Planner,
    }
}

/// `of` the environment variable VARIABLE. A variable that is not set or not Unicode counts as
/// none.
pub fn current() -> Role {
    of(std::env::var(VARIABLE).ok().as_deref())
}

#[cfg(test)]
mod tests {
    use super::{Role, of};

    #[test]
    fn no_value_or_an_empty_value_is_a_planner() {
        assert_eq!(of(None), Role::Planner);
        assert_eq!(of(Some("")), Role::Planner);
    }

    #[test]
    fn other_words_are_a_planner() {
        assert_eq!(of(Some("planner")), Role::Planner);
        assert_eq!(of(Some("workers")), Role::Planner);
    }

    #[test]
    fn worker_in_any_case_and_with_spaces_is_a_worker() {
        assert_eq!(of(Some("worker")), Role::Worker);
        assert_eq!(of(Some(" Worker ")), Role::Worker);
        assert_eq!(of(Some("WORKER")), Role::Worker);
    }
}
