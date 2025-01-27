use std::path::Path;

use actix_multipart::form::MultipartForm;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::{
    middlewares::auth_middleware::UserData, responses::general_error::GeneralError,
    validation_types::profile::add_image::UploadForm, AppState,
};

pub async fn add_image(
    req: HttpRequest,
    _: web::Data<AppState>,
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

    let file = form.file;

    let save_path = format!(
        "./uploaded_files/{}{}.png",
        user_data.user_id,
        file.file_name.unwrap()
    );
    let save_path = Path::new(&save_path);

    if let Err(e) = tokio::fs::create_dir_all(save_path.parent().unwrap()).await {
        return HttpResponse::InternalServerError().json(GeneralError {
            message: format!("Failed to create directory: {}", e),
        });
    }

    let mut f = match tokio::fs::File::create(save_path).await {
        Ok(file) => file,
        Err(_) => {
            return HttpResponse::InternalServerError().json(GeneralError {
                message: "Failed to create the file.".to_string(),
            });
        }
    };

    HttpResponse::Ok().json(user_data)
}
