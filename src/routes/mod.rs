use std::sync::Arc;

use crate::AppState;

use crate::entities::{
    notes::{CreateNote, Filter, Note, UpdateNote},
    users::{CreateUser, User},
};
use axum::{
    Json, Router,
    extract::{Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
};
use axum_cookie::CookieLayer;
use casbin::CoreApi;
use serde::Serialize;
use utoipa::{
    OpenApi,
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::routes::auth::AuthUser;

pub mod auth;
pub mod notes;
pub mod users;

async fn casbin_mv(
    State(state): State<Arc<AppState>>,
    AuthUser(claims): AuthUser,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let method = req.method().to_string();

    let allowed = state
        .enforcer
        .read()
        .await
        .enforce((claims.role.as_str(), path.as_str(), method.as_str()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !allowed {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

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
    components(schemas(User, CreateUser, Note, CreateNote, Filter, UpdateNote)),
    modifiers(&BearerAuth),
    security(("bearer_token" = []))
)]
pub struct ApiDoc;

struct BearerAuth;

impl utoipa::Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer_token",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}

pub async fn create_app(pool: Arc<AppState>) -> anyhow::Result<Router> {
    let public = Router::new()
        .nest("/users", users::public_router())
        .with_state(pool.clone());

    let protected = Router::new()
        .nest("/notes", notes::router())
        .nest("/users", users::protected_router(pool.clone()))
        .layer(from_fn_with_state(pool.clone(), casbin_mv))
        .with_state(pool.clone());

    let router = Router::new()
        .merge(public)
        .merge(protected)
        .layer(CookieLayer::default())
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

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, msg)
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, msg)
    }
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, msg)
    }
    pub fn unprocessable(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, msg)
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
    fn new(status_code: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            status_code,
            data: Json(Data {
                message: msg.into(),
            }),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status_code, self.data).into_response()
    }
}
