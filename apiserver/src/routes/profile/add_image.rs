use std::collections::BTreeSet;

use crate::{
    middlewares::auth_middleware::UserData, responses::general_error::GeneralError,
    validation_types::profile::add_image::UploadForm, AppState,
};
use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use cloudinary::upload::{OptionalParameters, Source, UploadResult};

pub async fn add_image(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    if req.extensions().get::<UserData>().is_none() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue talking to the database".to_string(),
        });
    }

    let extensions = req.extensions();
    let user_data = extensions.get::<UserData>().unwrap();

    let content_type = &form.file.content_type;
    if content_type.as_ref().unwrap().type_() != mime::IMAGE {
        return HttpResponse::BadRequest().json(GeneralError {
            message: "Not an image".to_string(),
        });
    }
    let profile_id_res = app_state.snow_flake.lock().unwrap().generate_id();

    if profile_id_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue generating profile id".to_string(),
        });
    }

    let transaction_res = app_state.database_connection_pool.begin().await;
    if transaction_res.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue starting the transaction".to_string(),
        });
    }

    let mut transaction = transaction_res.unwrap();

    let query_result = sqlx::query!(
        "INSERT INTO profile (id, user_id) VALUES ($1, $2)",
        *profile_id_res.as_ref().unwrap() as i64,
        user_data.user_id
    )
    .execute(&mut *transaction) // Dereference the transaction here
    .await;

    if query_result.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Failed to execute query".to_string(),
        });
    }

    let file = form.file;

    let options = BTreeSet::from([OptionalParameters::PublicId(format!(
        "gravatar/{}/{}",
        user_data.user_id,
        profile_id_res.unwrap()
    ))]);

    let cld_result = app_state
        .cloudinary
        .image(
            Source::Path(file.file.path().to_str().unwrap().into()),
            &options,
        )
        .await;
    if cld_result.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue writing to the cloud".to_string(),
        });
    }
    let commit_result = transaction.commit().await;
    if commit_result.is_err() {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: "Issue writing to the cloud".to_string(),
        });
    }

    let mut secure_url = "".to_owned();
    let cld_res = cld_result.unwrap();

    if let UploadResult::Response(val) = cld_res {
        secure_url = val.secure_url;
    }

    HttpResponse::Ok().json(secure_url)
}
