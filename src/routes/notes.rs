use crate::{
    db::notes::NoteRepo,
    entities::{
        notes::{CreateNote, CreateNoteRequest, Note, UpdateNote},
        repository::Repository,
    },
    routes::{AppError, AppState, Data, auth::AuthUser},
};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
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
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

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
    let note = NoteRepo::get_by_id(&state.pool, id)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    match note {
        Some(s) => Ok(Json(s)),
        None => Err(AppError {
            status_code: StatusCode::NOT_FOUND,
            data: Json(Data {
                message: String::from("Note not found"),
            }),
        }),
    }
}

#[utoipa::path(
    get,
    path = "/notes",
    responses(
        (status = 200, description = "List of notes", body = Vec<Note>),
    )
)]
pub async fn get_all(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
) -> Result<Json<Vec<Note>>, AppError> {
    let notes = NoteRepo::get_all(&state.pool)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    Ok(Json(notes))
}

#[utoipa::path(
    put,
    path = "/notes/{id}",
    params(("id" = i64, Path, description = "Note ID")),
    request_body = CreateNote,
    responses(
        (status = 200, description = "Note updated", body = Note),
        (status = 404, description = "Note not found"),
    )
)]
pub async fn update_note(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateNote>,
) -> Result<Json<Note>, AppError> {
    let updated_id = NoteRepo::update(&state.pool, id, &payload)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    Ok(Json(Note {
        id: updated_id,
        title: payload.title.clone(),
        description: payload.description.clone(),
        created_at: None,
        user: None,
    }))
}

#[utoipa::path(
    delete,
    path = "/notes/{id}",
    params(("id" = i64, Path, description = "Note ID")),
    responses(
        (status = 200, description = "note deleted", body = i64),
        (status = 404, description = "note not found"),
    )
)]
pub async fn delete_note(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    Path(id): Path<i64>,
) -> Result<Json<i64>, AppError> {
    let deleted_id = NoteRepo::delete(&state.pool, id)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    match deleted_id {
        Some(id) => Ok(Json(id)),
        None => Err(AppError {
            status_code: StatusCode::NOT_FOUND,
            data: Json(Data {
                message: String::from("Note not found"),
            }),
        }),
    }
}
