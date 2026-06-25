use std::sync::Arc;

use crate::AppState;
use crate::entities::{
    notes::{CreateNote, Note},
    users::{CreateUser, User},
};
use crate::routes::casbin::casbin_mv;
use axum::middleware::from_fn_with_state;
use axum::{Json, Router, http::StatusCode, response::IntoResponse};
use axum_cookie::CookieLayer;
use serde::Serialize;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod auth;
pub mod casbin;
pub mod notes;
pub mod users;

#[derive(OpenApi)]
#[openapi(
    paths(
        users::create_user,
        users::get_user,
        users::get_all,
        users::update_user,
        users::delete_user,
        users::login,
        notes::create_note,
        notes::get_note,
        notes::get_all,
        notes::update_note,
        notes::delete_note,
    ),
    components(schemas(User, CreateUser, Note, CreateNote))
)]
pub struct ApiDoc;

pub async fn create_app(pool: Arc<AppState>) -> anyhow::Result<Router> {
    let router = Router::new()
        .nest("/users", users::router())
        .nest("/notes", notes::router())
        .layer(CookieLayer::default())
        .layer(from_fn_with_state(pool.clone(), casbin_mv))
        .with_state(pool)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    Ok(router)
}

#[derive(Serialize)]
pub struct Data {
    pub message: String,
}

pub struct AppError {
    pub status_code: StatusCode,
    pub data: Json<Data>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, self.data).into_response()
    }
}
