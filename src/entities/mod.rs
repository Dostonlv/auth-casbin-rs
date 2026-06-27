pub mod auth;
pub mod notes;
pub mod repository;
pub mod users;

pub(crate) trait Validate {
    fn validate(&self) -> Result<(), &'static str>;
}

pub(crate) fn is_valid_email(email: &str) -> bool {
    matches!(email.splitn(2, '@').collect::<Vec<_>>().as_slice(), [local, domain]
        if !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.'))
}
