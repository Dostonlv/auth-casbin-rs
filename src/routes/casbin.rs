use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use casbin::CoreApi;
use notes::AppState;

use crate::routes::auth::AuthUser;

pub async fn casbin_mv(
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
