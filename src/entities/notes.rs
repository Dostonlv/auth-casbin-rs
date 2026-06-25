use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utility_types::{Omit, Pick};
use utoipa::ToSchema;

use crate::entities::users::User;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, FromRow, ToSchema, Omit, Pick)]
#[omit(arg(ident = Note, fields(user_id), derive(Debug, Serialize, Deserialize, ToSchema)), forward_attrs())]
#[pick(arg(ident = CreateNote, fields(title, description, user_id), derive(Debug, Deserialize, ToSchema)), forward_attrs())]
#[pick(arg(ident = CreateNoteRequest, fields(title, description), derive(Debug, Deserialize, ToSchema)))]
#[pick(arg(ident = UpdateNote, fields(title, description, user_id), derive(Debug, Deserialize, ToSchema)))]
pub struct NoteSchema {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub created_at: Option<chrono::NaiveDateTime>,
    #[sqlx(flatten)]
    pub user: Option<User>,
    pub user_id: i64,
}
