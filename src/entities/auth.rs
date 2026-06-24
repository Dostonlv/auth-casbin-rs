use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Token {
    pub token: String,
    pub expire: i64, // unix
}

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
}
