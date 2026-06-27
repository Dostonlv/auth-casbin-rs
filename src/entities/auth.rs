use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct Token {
    pub token: String,
    pub expire: i64, // unix
}

#[derive(Deserialize, ToSchema)]
pub struct Login {
    pub email: String,
    pub password: String,
}

impl super::Validate for Login {
    fn validate(&self) -> Result<(), &'static str> {
        if self.email.is_empty() || self.password.is_empty() {
            return Err("email and password must not be empty");
        }
        if !super::is_valid_email(&self.email) {
            return Err("invalid email address");
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub role: String,
}
