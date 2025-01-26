use crate::AppState;
use actix_web::{web, HttpResponse, Responder};
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

    HttpResponse::Ok().json(crate::responses::done_message::GoodResponse {
        message: "yo".to_string(),
    })
}
