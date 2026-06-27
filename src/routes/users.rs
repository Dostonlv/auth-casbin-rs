use crate::{
    db::users::UserRepo,
    entities::{
        Validate,
        auth::{Claims, Login, Token},
        repository::Repository,
        users::{CreateUser, UpdateUser, User},
    },
    routes::{AppError, AppState, auth::AuthUser},
};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{Next, from_fn_with_state},
    response::Response,
    routing::{get, post},
};
use axum_cookie::CookieManager;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

async fn own_resource_mv(
    AuthUser(claims): AuthUser,
    Path(id): Path<i64>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if claims.sub != id && claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(next.run(req).await)
}

pub fn public_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_user))
        .route("/login", post(login))
}

pub fn protected_router(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let id_routes = Router::new()
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .layer(from_fn_with_state(state, own_resource_mv));

    Router::new()
        .route("/", get(get_all))
        .route("/logout", post(logout))
        .merge(id_routes)
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User created", body = User),
        (status = 422, description = "Validation error"),
    ),
    security(())
)]
#[axum::debug_handler]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(mut payload): Json<CreateUser>,
) -> Result<Json<User>, AppError> {
    payload.validate().map_err(AppError::unprocessable)?;

    let salt = SaltString::generate(&mut OsRng);
    let pswd = Argon2::default()
        .hash_password(payload.password.as_deref().unwrap().as_bytes(), &salt)
        .map_err(|e| AppError::internal(e.to_string()))?
        .to_string();
    payload.password = Some(pswd);

    let id = UserRepo::create(&state.pool, &payload)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?;

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
    UserRepo::get_by_id(&state.pool, id)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?
        .map(Json)
        .ok_or_else(|| AppError::not_found("user not found"))
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
    )
)]
pub async fn get_all(State(state): State<Arc<AppState>>) -> Result<Json<Vec<User>>, AppError> {
    UserRepo::get_all(&state.pool, &())
        .await
        .map(Json)
        .map_err(|e| AppError::bad_request(e.to_string()))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    request_body = UpdateUser,
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    let updated_id = UserRepo::update(&state.pool, id, &payload)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?;

    Ok(Json(User {
        id: updated_id,
        full_name: payload.full_name,
        email: payload.email,
        created_at: None,
        password: None,
        role: "user".to_owned(),
    }))
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "User deleted", body = i64),
        (status = 404, description = "User not found"),
    )
)]
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<i64>, AppError> {
    UserRepo::delete(&state.pool, id)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?
        .map(Json)
        .ok_or_else(|| AppError::not_found("user not found"))
}

#[utoipa::path(
    post,
    path = "/users/login",
    request_body = Login,
    responses(
        (status = 200, description = "Token issued", body = Token),
        (status = 401, description = "Invalid credentials"),
        (status = 422, description = "Validation error"),
    ),
    security(())
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Login>,
) -> Result<Json<Token>, AppError> {
    payload.validate().map_err(AppError::unprocessable)?;

    let user = User::get_by_email(&state.pool, &payload.email)
        .await
        .map_err(|e| AppError::bad_request(e.to_string()))?
        .ok_or_else(|| AppError::not_found("user not found"))?;

    Argon2::default()
        .verify_password(
            payload.password.as_bytes(),
            &PasswordHash::new(user.password.as_deref().unwrap()).unwrap(),
        )
        .map_err(|_| AppError::unauthorized("incorrect password"))?;

    let exp_secs = state.config.jwt_expires_time.parse::<i64>().unwrap();
    let expiration = OffsetDateTime::now_utc() + Duration::seconds(exp_secs);
    let claims = Claims {
        sub: user.id,
        exp: expiration.unix_timestamp(),
        iat: OffsetDateTime::now_utc().unix_timestamp(),
        iss: state.config.jwt_issuer.clone(),
        role: user.role,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::internal(e.to_string()))?;

    Ok(Json(Token {
        token,
        expire: expiration.unix_timestamp(),
    }))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    cookie: CookieManager,
) -> Result<(), AppError> {
    if let Some(cookie) = cookie.get("auth_token") {
        let jwt_expires_time = state.config.jwt_expires_time.parse::<u64>().unwrap();
        state
            .redis_adaptor
            .add_token_to_black_list(cookie.value(), jwt_expires_time)
            .map_err(|_| AppError::internal("failed to invalidate token"))?;
    }
    Ok(())
}
