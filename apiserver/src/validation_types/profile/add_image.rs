use actix_multipart::form::{tempfile::TempFile, MultipartForm};

#[derive(Debug, MultipartForm)]
pub struct UploadForm {
    #[multipart(limit = "100KB")]
    pub file: TempFile,
}
