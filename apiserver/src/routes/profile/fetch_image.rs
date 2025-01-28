use actix_web::{web, HttpResponse, Responder};

use crate::{models::user_model::UserFromDB, responses::general_error::GeneralError, AppState};

#[derive(serde::Deserialize)]
pub struct PathParams {
    pub email_hash: String,
}

pub async fn get_profile_image(
    app_state: web::Data<AppState>,
    path: web::Path<PathParams>,
) -> impl Responder {
    let user_from_db_res =
        sqlx::query_as::<_, UserFromDB>("select * from users where email_hash = $1")
            .bind(&path.email_hash)
            .fetch_optional(&app_state.database_connection_pool)
            .await;

    if user_from_db_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    if user_from_db_res.as_ref().unwrap().is_none() {
        return HttpResponse::NotFound().json(GeneralError {
            message: "Not found".to_string(),
        });
    }

    if user_from_db_res
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap()
        .active_photo_id
        == -1
    {
        return HttpResponse::BadRequest().json(GeneralError {
            message: "User has not kept any profile picture yet".to_string(),
        });
    }

    HttpResponse::Ok().json(user_from_db_res.unwrap().unwrap().active_photo_id)
}
