use crate::{
    db::users::UserRepo,
    entities::{
        repository::Repository,
        users::{CreateUser, User},
    },
    routes::{AppError, AppState, Data},
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
        .route("/", get(get_all).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User created", body = User),
        (status = 400, description = "Bad request"),
    )
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, AppError> {
    let id = UserRepo::create(&state.pool, &payload)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    Ok(Json(User {
        id,
        full_name: payload.full_name,
        email: payload.email,
        created_at: None,
        password: None,
        role: "user".to_owned(),
    }))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found"),
    )
)]
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<User>, AppError> {
    let user = UserRepo::get_by_id(&state.pool, id)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    match user {
        Some(s) => Ok(Json(s)),
        None => Err(AppError {
            status_code: StatusCode::NOT_FOUND,
            data: Json(Data {
                message: String::from("User not found"),
            }),
        }),
    }
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
    )
)]
pub async fn get_all(State(state): State<Arc<AppState>>) -> Result<Json<Vec<User>>, AppError> {
    let users = UserRepo::get_all(&state.pool)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    Ok(Json(users))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    request_body = CreateUser,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<User>, AppError> {
    let updated_id = UserRepo::update(&state.pool, id, &payload)
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST,
            data: Json(Data {
                message: format!("{:?}", err),
            }),
        })?;

    Ok(Json(User {
        id: updated_id,
        full_name: payload.full_name,
        email: payload.email,
        created_at: None,
        password: None,
        role: "user".to_string(),
    }))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "user deleted", body = i64),
        (status = 404, description = "User not found"),
    )
)]
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<i64>, AppError> {
    let deleted_id = UserRepo::delete(&state.pool, id)
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
                message: String::from("User not found"),
            }),
        }),
    }
}
