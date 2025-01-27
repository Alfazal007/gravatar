use actix_web::{
    cookie::{Cookie, SameSite},
    web, HttpResponse, Responder,
};
use redis::Commands;
use validator::Validate;

#[derive(serde::Serialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "userId")]
    user_id: i64,
}

use crate::{
    models::user_model::UserFromDBWithPassword, responses::general_error::GeneralError, AppState,
};

pub async fn login_user(
    data: web::Data<AppState>,
    sign_in_data: web::Json<crate::validation_types::user::signin::SigninData>,
) -> impl Responder {
    if let Err(e) = sign_in_data.validate() {
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

    // find the user from the database
    let user_from_db_res =
        sqlx::query_as::<_, UserFromDBWithPassword>("select * from users where email = $1")
            .bind(&sign_in_data.0.email)
            .fetch_optional(&data.database_connection_pool)
            .await;
    if user_from_db_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    if user_from_db_res.as_ref().unwrap().is_none() {
        return HttpResponse::NotFound().json(GeneralError {
            message: "User not found in the database".to_string(),
        });
    }

    // compare passwords
    let valid_password_res = bcrypt::verify(
        &sign_in_data.0.password,
        &user_from_db_res
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .password,
    );

    if valid_password_res.is_err() || !valid_password_res.unwrap() {
        return HttpResponse::BadRequest().json(GeneralError {
            message: "Incorrect password".to_string(),
        });
    }

    let token_res = crate::helpers::generate_token::generate_token(
        &sign_in_data.email,
        user_from_db_res.as_ref().unwrap().as_ref().unwrap().id,
        &data.access_token_secret,
    );

    if token_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue generating the token".to_string(),
        });
    }

    let cookie1 = Cookie::build("accessToken", token_res.as_ref().unwrap())
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::None)
        .finish();

    let cookie2 = Cookie::build(
        "userId",
        format!(
            "{}",
            user_from_db_res.as_ref().unwrap().as_ref().unwrap().id
        ),
    )
    .path("/")
    .secure(true)
    .http_only(true)
    .same_site(SameSite::None)
    .finish();

    let redis_conn = data.redis_conn.get();
    if redis_conn.is_ok() {
        let _: () = redis_conn
            .unwrap()
            .set(
                format!(
                    "auth:{}",
                    user_from_db_res.as_ref().unwrap().as_ref().unwrap().id
                ),
                token_res.as_ref().unwrap(),
            )
            .unwrap();
    }

    HttpResponse::Ok()
        .cookie(cookie1)
        .cookie(cookie2)
        .json(LoginResponse {
            user_id: user_from_db_res.unwrap().unwrap().id,
            access_token: token_res.unwrap(),
        })
}
