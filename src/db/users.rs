use sqlx::{Error, SqlitePool};

use crate::entities::{
    repository::Repository,
    users::{CreateUser, UpdateUser, User},
};

impl User {
    pub async fn get_by_email(pool: &SqlitePool, email: &str) -> Result<Option<Self>, Error> {
        sqlx::query_as!(
            User,
            r#"
            SELECT id, full_name, email, password, created_at  FROM users WHERE email = $1
        "#,
            email
        )
        .fetch_optional(pool)
        .await
    }
    pub async fn update_password(&self, pool: &SqlitePool) -> Result<i64, Error> {
        sqlx::query_scalar!(
            "UPDATE  users SET password = $1 WHERE id=$2 and email = $3  RETURNING id",
            self.password,
            self.id,
            &self.email
        )
        .fetch_one(pool)
        .await
    }
}
pub struct UserRepo;

impl Repository for UserRepo {
    type Model = User;
    type CreateDto = CreateUser;
    type UpdateDto = UpdateUser;

    async fn create(pool: &SqlitePool, dto: &Self::CreateDto) -> Result<i64, Error> {
        sqlx::query_scalar!(
            r#"INSERT INTO users (full_name, email, password)
            VALUES ($1, $2, $3) RETURNING id"#,
            dto.full_name,
            dto.email,
            dto.password
        )
        .fetch_one(pool)
        .await
    }

    async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self::Model>, Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, full_name, email, '' as password,created_at
            FROM users WHERE id = $1"#,
            id
        )
        .fetch_optional(pool)
        .await
    }

    async fn get_all(pool: &SqlitePool) -> Result<Vec<Self::Model>, Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, full_name, email, '' as password,created_at
            FROM users"#,
        )
        .fetch_all(pool)
        .await
    }

    async fn update(pool: &SqlitePool, id: i64, dto: &Self::UpdateDto) -> Result<i64, Error> {
        sqlx::query_scalar!(
            r#"UPDATE users
            SET full_name = $1, email = $2
            WHERE id = $3 RETURNING id"#,
            dto.full_name,
            dto.email,
            id
        )
        .fetch_one(pool)
        .await
    }

    async fn delete(pool: &SqlitePool, id: i64) -> Result<Option<i64>, Error> {
        sqlx::query_scalar!(r#"DELETE FROM users WHERE id = $1 RETURNING id"#, id)
            .fetch_optional(pool)
            .await
    }
}
