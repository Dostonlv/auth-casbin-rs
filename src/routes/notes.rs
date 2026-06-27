use crate::{
    db::notes::NoteRepo,
    entities::{
        notes::{
            CreateNote, CreateNoteRequest, Filter, FilterSchema, Note, UpdateNote, UpdateNoteSchema,
        },
        repository::Repository,
    },
    routes::{AppError, AppState, auth::AuthUser},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create_note))
        .route("/{id}", get(get_note).put(update_note).delete(delete_note))
}

#[utoipa::path(
    post,
    path = "/notes",
    request_body = CreateNote,
    responses(
        (status = 200, description = "Note created", body = Note),
        (status = 400, description = "Bad request"),
    )
)]
#[axum::debug_handler]
pub async fn create_note(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Json(CreateNoteRequest { title, description }): Json<CreateNoteRequest>,
) -> Result<Json<Note>, AppError> {
    let note = CreateNote {
        user_id: claims.sub,
        title,
        description,
    };

    let id = NoteRepo::create(&state.pool, &note)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?;

    Ok(Json(Note {
        id,
        title: note.title,
        description: note.description,
        created_at: None,
        user: None,
    }))
}

#[utoipa::path(
    get,
    path = "/notes/{id}",
    params(("id" = i64, Path, description = "Note ID")),
    responses(
        (status = 200, description = "Note found", body = Note),
        (status = 404, description = "Note not found"),
    )
)]
pub async fn get_note(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<Note>, AppError> {
    NoteRepo::get_by_owner(&state.pool, id, claims.sub)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?
        .map(Json)
        .ok_or_else(|| AppError::not_found("note not found"))
}

#[utoipa::path(
    get,
    path = "/notes",
    params(Filter),
    responses(
        (status = 200, description = "List of notes", body = Vec<Note>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Query(filter): Query<Filter>,
) -> Result<Json<Vec<Note>>, AppError> {
    let filter = FilterSchema {
        page: filter.page,
        limit: filter.limit,
        user_id: claims.sub,
    };

    NoteRepo::get_all(&state.pool, &filter)
        .await
        .map(Json)
        .map_err(|e| AppError::bad_request(e.to_string()))
}

#[utoipa::path(
    put,
    path = "/notes/{id}",
    params(("id" = i64, Path, description = "Note ID")),
    request_body = UpdateNote,
    responses(
        (status = 200, description = "Note updated", body = Note),
        (status = 404, description = "Note not found"),
    )
)]
pub async fn update_note(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i64>,
    Json(UpdateNote { title, description }): Json<UpdateNote>,
) -> Result<Json<Note>, AppError> {
    let payload = UpdateNoteSchema {
        title,
        description,
        user_id: claims.sub,
    };

    let updated_id = NoteRepo::update(&state.pool, id, &payload)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?;

    Ok(Json(Note {
        id: updated_id,
        title: payload.title,
        description: payload.description,
        created_at: None,
        user: None,
    }))
}

#[utoipa::path(
    delete,
    path = "/notes/{id}",
    params(("id" = i64, Path, description = "Note ID")),
    responses(
        (status = 200, description = "Note deleted", body = i64),
        (status = 404, description = "Note not found"),
    )
)]
pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<i64>, AppError> {
    NoteRepo::delete_owned(&state.pool, id, claims.sub)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?
        .map(Json)
        .ok_or_else(|| AppError::not_found("note not found"))
}
