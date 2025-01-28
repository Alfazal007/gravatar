use validator::Validate;

#[derive(Validate, serde::Deserialize)]
pub struct UpdateProfileData {
    pub profile_id: i64,
}
