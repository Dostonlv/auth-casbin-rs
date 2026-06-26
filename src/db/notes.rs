use sqlx::{Error, SqlitePool};

use crate::entities::{
    notes::{CreateNote, FilterSchema, Note, UpdateNoteSchema}, repository::Repository, users::User,
};

pub struct NoteRepo;

impl Repository for NoteRepo {
    type Model = Note;
    type CreateDto = CreateNote;
    type UpdateDto = UpdateNoteSchema;
    type Filter = FilterSchema;

    async fn create(pool: &SqlitePool, dto: &Self::CreateDto) -> Result<i64, Error> {
        sqlx::query_scalar!(
            r#"INSERT INTO notes (title, description, user_id)
            VALUES ($1, $2, $3) RETURNING id"#,
            dto.title,
            dto.description,
            dto.user_id
        )
        .fetch_one(pool)
        .await?
        .ok_or(Error::RowNotFound)
    }
    async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self::Model>, Error> {
        let row = sqlx::query!(
            r#"
                SELECT 
                    n.id, 
                    n.title, 
                    n.description, 
                    n.created_at, 
                    u.id as user_id, 
                    u.full_name, 
                    u.email,
                    u.role
                FROM notes n 
                INNER JOIN users u ON n.user_id = u.id 
                WHERE n.id = $1
                "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        let note = row.map(|n| Note {
            id,
            title: n.title,
            description: n.description,
            created_at: n.created_at,
            user: Some(User {
                id: n.user_id,
                full_name: n.full_name,
                email: n.email,
                password: None,
                created_at: None,
                role: n.role,
            }),
        });
        Ok(note)
    }

    async fn get_all(pool: &SqlitePool,  _filter: &Self::Filter) -> Result<Vec<Self::Model>, Error> {
        let offset = (_filter.page-1)* _filter.limit;
        
        let rows = sqlx::query!(
            r#"
                SELECT 
                    n.id, 
                    n.title, 
                    n.description, 
                    n.created_at, 
                    u.id as user_id, 
                    u.full_name, 
                    u.email,
                    u.role
                FROM notes n 
                INNER JOIN users u ON n.user_id = u.id
                WHERE n.user_id = $1 LIMIT $2 OFFSET $3
                "#,
                _filter.user_id,
                _filter.limit,
                offset
        )
        .fetch_all(pool)
        .await?;
        let notes = rows
            .into_iter()
            .map(|n| Note {
                id: n.id,
                title: n.title,
                description: n.description,
                created_at: n.created_at,
                user: Some(User {
                    id: n.user_id,
                    full_name: n.full_name,
                    email: n.email,
                    password: None,
                    created_at: None,
                    role: n.role,
                }),
            })
            .collect();
        Ok(notes)
    }

    async fn update(pool: &SqlitePool, id: i64, dto: &Self::UpdateDto) -> Result<i64, Error> {
        sqlx::query_scalar!(
            r#"UPDATE notes 
        SET title = $1, description = $2, user_id = $3
        WHERE id = $4 and user_id = $5
        RETURNING id"#,
            dto.title,
            dto.description,
            dto.user_id,
            id,
            dto.user_id
        )
        .fetch_one(pool)
        .await?
        .ok_or(Error::RowNotFound)
    }

    async fn delete(pool: &SqlitePool, id: i64) -> Result<Option<i64>, Error> {
        sqlx::query_scalar!(r#"DELETE FROM notes WHERE id = $1 RETURNING id"#, id)
            .fetch_optional(pool)
            .await
    }
}
