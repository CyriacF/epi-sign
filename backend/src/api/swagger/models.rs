use crate::api::auth;
use crate::api::sign;
use crate::api::users;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::endpoints::register,
        auth::endpoints::login,
        auth::endpoints::logout,
        users::endpoints::get_me,
        users::endpoints::get_users,
        users::endpoints::update_jwt,
        users::endpoints::update_user,
        users::endpoints::save_signature,
        users::endpoints::get_signatures,
        users::endpoints::delete_signature,
        users::endpoints::delete_account,
        sign::endpoints::sign,
        sign::endpoints::status
    ),
    tags(
        (name = "Auth", description = "Authentication related endpoints"),
        (name = "Users", description = "User management endpoints"),
        (name = "Sign", description = "Epitech signing endpoints")
    )
)]
pub struct Swagger;
