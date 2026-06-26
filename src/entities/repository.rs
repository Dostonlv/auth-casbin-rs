use sqlx::{Error, SqlitePool};

pub trait Repository {
    type Model;
    type CreateDto;
    type UpdateDto;
    type Filter;

    async fn create(pool: &SqlitePool, dto: &Self::CreateDto) -> Result<i64, Error>;
    async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Self::Model>, Error>;
    async fn get_all(pool: &SqlitePool, filter: &Self::Filter) -> Result<Vec<Self::Model>, Error>;
    async fn update(pool: &SqlitePool, id: i64, dto: &Self::UpdateDto) -> Result<i64, Error>;
    async fn delete(pool: &SqlitePool, id: i64) -> Result<Option<i64>, Error>;
}
