use crate::{responses::general_error::GeneralError, AppState};
use actix_web::{web, HttpResponse, Responder};
use bcrypt::hash;
use validator::Validate;

pub async fn create_user(
    data: web::Data<AppState>,
    sign_up_data: web::Json<crate::validation_types::user::signup::SignupData>,
) -> impl Responder {
    if let Err(e) = sign_up_data.validate() {
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

    // check if user with same email exists
    let user_with_same_email_result = sqlx::query_as::<_, crate::models::user_model::UserFromDB>(
        "select * from users where email=$1",
    )
    .bind(&sign_up_data.0.email)
    .fetch_optional(&data.database_connection_pool)
    .await;

    if user_with_same_email_result.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    if user_with_same_email_result.unwrap().is_some() {
        return HttpResponse::BadRequest().json(GeneralError {
            message: "User with same email exists in the database".to_string(),
        });
    }

    let user_id_result: Result<u64, String>;
    {
        user_id_result = data.snow_flake.lock().unwrap().generate_id();
    }

    if user_id_result.is_err() {
        return HttpResponse::BadRequest().json(GeneralError {
            message: "Issue generating the id".to_string(),
        });
    }

    let digest = md5::compute(&sign_up_data.0.email);
    let email_hash_hex = hex::encode(digest.0);

    let password_hash_result = hash(&sign_up_data.0.password, 12);

    if password_hash_result.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue hashing the password".to_string(),
        });
    }

    let new_user_create_result = sqlx::query_as::<_, crate::models::user_model::UserFromDB>(
        "insert into users(email, password, email_hash, id) values(
			$1, $2, $3, $4) returning *
		",
    )
    .bind(&sign_up_data.0.email)
    .bind(password_hash_result.unwrap())
    .bind(email_hash_hex)
    .bind(user_id_result.unwrap() as i64)
    .fetch_optional(&data.database_connection_pool)
    .await;

    if new_user_create_result.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    if new_user_create_result.as_ref().unwrap().is_none() {
        return HttpResponse::BadRequest().json(GeneralError {
            message: "User not created".to_string(),
        });
    }

    HttpResponse::Ok().json(new_user_create_result.unwrap())
}
