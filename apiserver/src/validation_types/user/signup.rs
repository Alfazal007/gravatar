use validator::Validate;

#[derive(Validate, serde::Deserialize)]
pub struct SignupData {
    #[validate(email)]
    pub email: String,
    #[validate(length(
        min = 6,
        max = 20,
        message = "Password should be between 6 and 20 length"
    ))]
    pub password: String,
}
