use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use http::StatusCode;

#[cfg(not(debug_assertions))]
use tower_cookies::cookie::SameSite;
use tower_cookies::cookie::time;
use tower_cookies::{Cookie, Cookies};
use tracing::error;
use validator::Validate;

use crate::api::auth::{JwtClaims, LoginPayload, RegisterPayload};
use crate::api::users::{User, create_user, get_user_by_username, user_exists};
use crate::misc::GlobalState;

#[utoipa::path(
    post,
    path = "/api/auth/register",
    description = "Register a new user",
    request_body = RegisterPayload,
    responses(
        (status = 201, description = "Registered successfully"),
        (status = BAD_REQUEST, description = "Invalid request payload"),
        (status = CONFLICT, description = "User already exists"),
    ),
    tag = "Auth"
)]
pub async fn register(
    State(state): State<GlobalState>,
    Json(payload): Json<RegisterPayload>,
) -> impl IntoResponse {
    if payload.validate().is_err() {
        return (StatusCode::BAD_REQUEST).into_response();
    }

    if payload.key != state.register_key {
        return (StatusCode::UNAUTHORIZED).into_response();
    }

    match user_exists(&state, &payload) {
        Ok(_) => (),
        Err(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        )) => {
            return (StatusCode::CONFLICT).into_response();
        }
        Err(e) => {
            error!("Failed to check if user exists: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    }

    match create_user(&state, payload.into()) {
        Ok(_) => (StatusCode::CREATED).into_response(),
        Err(e) => {
            error!("Failed to create user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR).into_response()
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    description = "Login a user",
    request_body = LoginPayload,
    responses(
        (status = 200, description = "Logged in successfully"),
        (status = BAD_REQUEST, description = "Invalid request payload"),
        (status = UNAUTHORIZED, description = "Invalid credentials"),
    ),
    tag = "Auth"
)]
pub async fn login(
    State(state): State<GlobalState>,
    cookies: Cookies,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    if payload.validate().is_err() {
        return (StatusCode::BAD_REQUEST).into_response();
    }

    let user: User = match get_user_by_username(&state, &payload.username) {
        Ok(Some(user)) => user,
        Ok(None) => {
            return (StatusCode::UNAUTHORIZED).into_response();
        }
        Err(e) => {
            error!("Failed to get user by username: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    };

    if !user.verify_password(&payload.password) {
        return (StatusCode::UNAUTHORIZED).into_response();
    }

    let jwt = JwtClaims::from(user);
    let jwt = match jwt.to_string() {
        Ok(jwt) => jwt,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR).into_response(),
    };

    let mut cookie = Cookie::new("auth", jwt);

    #[cfg(not(debug_assertions))]
    cookie.set_secure(true);
    #[cfg(not(debug_assertions))]
    cookie.set_same_site(SameSite::None);
    cookie.set_expires(time::OffsetDateTime::now_utc() + time::Duration::days(7));
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    (StatusCode::OK).into_response()
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully"),
        (status = UNAUTHORIZED, description = "Invalid credentials"),
    ),
    security(
        ("cookieAuth" = [])
    ),
    tag = "Auth",
)]
pub async fn logout(cookies: Cookies) -> impl IntoResponse {
    let mut cookie = Cookie::new("auth", "");

    #[cfg(not(debug_assertions))]
    cookie.set_secure(true);
    #[cfg(not(debug_assertions))]
    cookie.set_same_site(SameSite::None);
    cookie.set_expires(time::OffsetDateTime::UNIX_EPOCH);
    cookie.set_path("/");
    cookie.set_http_only(true);
    cookies.add(cookie);

    (StatusCode::OK).into_response()
}
