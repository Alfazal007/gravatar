use sqlx::prelude::FromRow;

#[derive(FromRow, serde::Deserialize, Debug, serde::Serialize)]
pub struct UserFromDB {
    pub id: i64,
    pub email: String,
    pub email_hash: Option<String>,
    pub active_photo_id: i32,
}

#[derive(FromRow, serde::Deserialize, Debug, serde::Serialize)]
pub struct UserFromDBWithPassword {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub email_hash: Option<String>,
    pub active_photo_id: i32,
}
