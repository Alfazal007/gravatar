use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use validator::Validate;

use crate::{
    middlewares::auth_middleware::UserData,
    models::{profile_model::ProfileFromDB, user_model::UserFromDB},
    responses::general_error::GeneralError,
    AppState,
};

pub async fn update_profile_image(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    profile_data: web::Json<crate::validation_types::profile::update_profile::UpdateProfileData>,
) -> impl Responder {
    if let Err(e) = profile_data.validate() {
        let mut validation_errors: Vec<String> = Vec::new();
        for (_, err) in e.field_errors().iter() {
            if let Some(message) = &err[0].message {
                validation_errors.push(message.clone().into_owned());
            }
        }
        if validation_errors.is_empty() {
            validation_errors.push("Invalid email".to_string())
        }
        return HttpResponse::BadRequest().json(
            crate::responses::validation_error::ValidationErrorsToBeReturned {
                errors: validation_errors,
            },
        );
    }

    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    // check if profile exists in the database
    let profile_exists_res =
        sqlx::query_as::<_, ProfileFromDB>("select * from profile where user_id=$1 and id=$2")
            .bind(user_data.user_id)
            .bind(profile_data.0.profile_id)
            .fetch_optional(&app_state.database_connection_pool)
            .await;

    if profile_exists_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    if profile_exists_res.unwrap().is_none() {
        return HttpResponse::NotFound().json(GeneralError {
            message: "Profile not found".to_string(),
        });
    }

    let updated_user_res = sqlx::query_as::<_, UserFromDB>(
        "update users set active_photo_id=$1 where id=$2 returning *",
    )
    .bind(profile_data.0.profile_id)
    .bind(user_data.user_id)
    .fetch_optional(&app_state.database_connection_pool)
    .await;

    if updated_user_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue updating the database".to_string(),
        });
    }

    HttpResponse::Ok().json(())
}
