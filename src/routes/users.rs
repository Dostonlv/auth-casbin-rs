use crate::{
    db::users::UserRepo,
    entities::{
        auth::{Claims, Login, Token},
        repository::Repository,
        users::{CreateUser, User},
    },
    routes::{AppError, AppState, Data},
};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router, extract::{Path, State}, http::{self, StatusCode}, routing::{get, post},
};
use axum_cookie::CookieManager;
use jsonwebtoken::{EncodingKey, Header, encode};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create_user))
        .route("/{id}", get(get_user).put(update_user).delete(delete_user))
        .route("/login", post(login))
        .route("/logout", post(logout))
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
    Json(mut payload): Json<CreateUser>,
) -> Result<Json<User>, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pswd = argon2
        .hash_password(payload.password.to_owned().as_bytes(), &salt)
        .unwrap()
        .to_string();
    payload.password = pswd;
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
        role: "user".to_string(), // use anyhow::Ok;
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

#[utoipa::path(
    post,
    path = "/users/login",
     request_body = Login,
    responses(
        (status = 200, description = "token given", body = Token),
        (status = 404, description = "Error"),
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Login>,
) -> Result<Json<Token>, AppError> {
    if payload.email.is_empty() && payload.password.is_empty() {
        return Err(AppError {
            status_code: StatusCode::BAD_REQUEST.into(),
            data: Json(Data {
                message: "email or password is not empty".to_string(),
            }),
        });
    }

    let user_request = User::get_by_email(&state.pool, payload.email.as_str())
        .await
        .map_err(|err| AppError {
            status_code: StatusCode::BAD_REQUEST.into(),
            data: Json(Data {
                message: err.to_string(),
            }),
        })?;

    let Some(user) = user_request else {
        return Err(AppError {
            status_code: StatusCode::NOT_FOUND.into(),
            data: Json(Data {
                message: String::from("user not found"),
            }),
        });
    };

    let verify = Argon2::default().verify_password(
        payload.password.as_bytes(),
        &PasswordHash::new(user.password.unwrap().as_str()).unwrap(),
    );

    if verify.is_err() {
        return Err(AppError {
            status_code: StatusCode::UNAUTHORIZED.into(),
            data: Json(Data {
                message: String::from("password is incorrect"),
            }),
        });
    }

    let exp_time = &state.config.jwt_expires_time.parse::<i64>().unwrap();
    let expiration = OffsetDateTime::now_utc() + Duration::seconds(*exp_time);
    let claims = Claims {
        sub: user.id.to_string(),
        exp: expiration.unix_timestamp(),
        iat: OffsetDateTime::now_utc().unix_timestamp(),
        iss: (&state.config.jwt_issuer).to_string(),
    };
    let secret = &state.config.jwt_secret.as_bytes();
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .unwrap();

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
        let token = cookie.value();
        let jwt_expires_time = state.config.jwt_expires_time.parse::<u64>().unwrap();
        let _ = state
            .redis_adaptor
            .add_token_to_black_list(token, jwt_expires_time)
            .map_err(|_| AppError {
                status_code: StatusCode::INTERNAL_SERVER_ERROR.into(),
                data: Json(Data { message: String::from("error while adding token to redis blacklist") }),
            });
    }
    Ok(())
}
