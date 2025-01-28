use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::{
    middlewares::auth_middleware::UserData, models::profile_model::AllProfiles,
    responses::general_error::GeneralError, AppState,
};

pub async fn get_imgages(req: HttpRequest, app_state: web::Data<AppState>) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }
    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    let profile_pics_res = sqlx::query_as::<_, AllProfiles>(
        "select COALESCE(array_agg(id), '{}') as id from profile where user_id=$1",
    )
    .bind(user_data.user_id)
    .fetch_optional(&app_state.database_connection_pool)
    .await;

    if profile_pics_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    if profile_pics_res.as_ref().unwrap().as_ref().is_none() {
        return HttpResponse::NotFound().json(());
    }

    HttpResponse::Ok().json(profile_pics_res.unwrap())
}
