use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::entities::users::User;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, FromRow, ToSchema)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    #[sqlx(flatten)]
    pub user: Option<User>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateNote {
    pub title: String,
    pub description: String,
    pub user_id: i64,
}

pub type UpdateNote = CreateNote;
