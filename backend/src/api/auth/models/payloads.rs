use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, Clone, Debug, Validate, ToSchema)]
pub struct LoginPayload {
    #[schema(example = "antoine")]
    pub username: String,
    #[validate(length(
        min = 6,
        message = "Le mot de passe doit contenir au moins 6 caractères"
    ))]
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Deserialize, Clone, Debug, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegisterPayload {
    #[validate(length(
        min = 3,
        max = 50,
        message = "Le nom d'utilisateur doit contenir entre 3 et 50 caractères"
    ))]
    #[schema(example = "antoine")]
    pub username: String,

    #[validate(length(
        min = 6,
        message = "Le mot de passe doit contenir au moins 6 caractères"
    ))]
    #[schema(example = "password123")]
    pub password: String,
    
    #[schema(example = "key")]
    pub key: String
}
