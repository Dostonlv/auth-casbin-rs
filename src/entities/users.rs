use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, FromRow, ToSchema, Default)]
pub struct User {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub password: Option<String>,
    pub role: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUser {
    pub full_name: String,
    pub email: String,
    pub password: Option<String>,
}

pub type UpdateUser = CreateUser;

#[derive(Debug, PartialEq, Clone)]
pub enum Role {
    User,
    Admin,
    Publisher,
}

impl Role {
    pub fn check_role(role: &str) -> Result<Self, &'static str> {
        match role {
            "user" | "User" => Ok(Role::User),
            "admin" | "Admin" => Ok(Role::Admin),
            _ => Err("Unknown role"),
        }
    }
}
