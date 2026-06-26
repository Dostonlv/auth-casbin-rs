use std::sync::Arc;

use axum::{
    RequestPartsExt,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use notes::AppState;

use crate::entities::auth::Claims;

pub struct AuthUser(pub Claims);

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "token not found"))?;
       let is_blacklisted = state
            .redis_adaptor
            .is_token_in_black_list(bearer.token().to_string())
            .unwrap_or(true);

        if is_blacklisted {
            return Err((StatusCode::UNAUTHORIZED, "Token is invalidated"));
        }
        let data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| (StatusCode::UNAUTHORIZED, "token is not valid or expired"))?;

        Ok(AuthUser(data.claims))
    }
}
