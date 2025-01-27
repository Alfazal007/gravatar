use sqlx::prelude::FromRow;

#[derive(FromRow, serde::Deserialize, Debug, serde::Serialize)]
pub struct ProfileFromDB {
    pub id: i64,
    pub user_id: i64,
}
