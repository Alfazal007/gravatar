use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub email: String,
    pub user_id: i64,
    pub exp: usize,
}

pub fn generate_token(
    email: &str,
    user_id: i64,
    access_token_secret: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let claims = Claims {
        user_id,
        email: email.to_string(),
        exp: (SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 86400) as usize,
    };
    let header = jsonwebtoken::Header::default();
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(access_token_secret.as_ref()),
    )?;
    Ok(token)
}
