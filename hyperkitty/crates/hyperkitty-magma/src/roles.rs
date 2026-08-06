#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Router,
    Expert,
    Validator,
    Observer,
    System,
}

pub fn derive_role_from_predicates(_predicates: &[String]) -> Option<Role> {
    Some(Role::Observer)
}
