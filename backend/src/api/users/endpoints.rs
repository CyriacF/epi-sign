use axum::{Json, extract::State, response::IntoResponse};
use base64::{Engine as _, engine::general_purpose};
use chrono::DateTime;
use http::StatusCode;
use serde_json::Value;
use tracing::{info, error};

use crate::{
    api::{
        auth::{JwtClaims, hash_password},
        users::{
            User, get_user_by_id, get_user_by_username,
            models::{JwtPayload, PublicUserResponse, UpdateUserPayload, SaveSignaturePayload, UserSignature},
            services::{get_all_users, update_user_jwt, add_user_signature, get_user_signatures, delete_user_signature},
        },
    },
    misc::GlobalState,
};

#[utoipa::path(
    get,
    path = "/api/users/me",
    description = "Get the current user's information",
    responses(
        (status = 200, description = "User information retrieved successfully", body = User),
        (status = 404, description = "User not found"),
        (status= 401, description = "Unauthorized - Invalid or missing JWT token")
    ),
    tag = "Users"
)]
pub async fn get_me(State(state): State<GlobalState>, jwt: JwtClaims) -> impl IntoResponse {
    let mut user = match get_user_by_id(&state, &jwt.sub) {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching user").into_response(),
    };
    if let Ok(sigs) = get_user_signatures(&state, &user.id) {
        if let Some(first) = sigs.first() {
            user.signature_manuscrite = Some(first.signature_data.clone());
        }
    }
    (StatusCode::OK, Json(user)).into_response()
}

#[utoipa::path(
    post,
    path = "/api/users/me/update-jwt",
    description = "Update the JWT for the current user",
    request_body = JwtPayload,
    responses(
        (status = 200, description = "JWT updated successfully"),
        (status = 400, description = "Invalid JWT format or payload"),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Users"
)]
pub async fn update_jwt(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Json(jwt_payload): Json<JwtPayload>,
) -> impl IntoResponse {
    let parts: Vec<&str> = jwt_payload.jwt.split('.').collect();

    if parts.len() != 3 {
        return (StatusCode::BAD_REQUEST, "Invalid JWT format").into_response();
    }

    let decoded = match general_purpose::URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(decoded) => decoded,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JWT payload").into_response(),
    };

    let exp = match serde_json::from_slice::<Value>(&decoded) {
        Ok(payload) => payload.get("exp").cloned(),
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid JWT payload format").into_response(),
    };

    let exp = match exp {
        Some(exp) => match exp.as_i64() {
            Some(exp) => exp,
            None => {
                return (StatusCode::BAD_REQUEST, "Expiration time is not an integer")
                    .into_response();
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Expiration time not found in JWT payload",
            )
                .into_response();
        }
    };

    if exp <= 0 {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid expiration time in JWT payload",
        )
            .into_response();
    }

    let exp_datetime = match DateTime::from_timestamp(exp, 0) {
        Some(datetime) => datetime,
        None => return (StatusCode::BAD_REQUEST, "Invalid timestamp").into_response(),
    };

    let exp_naive = exp_datetime.naive_utc();

    match update_user_jwt(&state, jwt_user.sub, &jwt_payload.jwt, exp_naive) {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR).into_response();
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/users",
    description = "Get all users",
    responses(
        (status = 200, description = "Users retrieved successfully", body = Vec<PublicUserResponse>),
        (status = 401 , description = "Unauthorized"),
    ),
    tag = "Users"
)]
pub async fn get_users(State(state): State<GlobalState>) -> impl IntoResponse {
    let users = match get_all_users(&state) {
        Ok(users) => users,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching users").into_response();
        }
    };

    if users.is_empty() {
        return (StatusCode::NOT_FOUND, "No users found").into_response();
    }

    let public_users: Vec<PublicUserResponse> = users
        .into_iter()
        .map(|user| PublicUserResponse::from(user))
        .collect();
    (StatusCode::OK, Json(public_users)).into_response()
}

#[utoipa::path(
    patch,
    path = "/api/users/me",
    description = "Update the current user's information",
    request_body = UpdateUserPayload,
    responses(
        (status = 200, description = "User updated successfully"),
        (status = 400, description = "Bad request - Invalid payload"),
        (status = 401, description = "Unauthorized - Invalid or missing JWT token"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users"
)]
pub async fn update_user(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Json(payload): Json<UpdateUserPayload>,
) -> impl IntoResponse {
    let mut user = match get_user_by_id(&state, &jwt_user.sub) {
        Ok(Some(user)) => user,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(_) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
    };

    if let Some(newpassword) = payload.new_password {
        if let Some(old_password) = payload.old_password {
            if !user.verify_password(&old_password) {
                return (StatusCode::UNAUTHORIZED, "Old password is incorrect").into_response();
            }
            user.password_hash = hash_password(&newpassword);
        } else {
            return (StatusCode::BAD_REQUEST, "Old password is required").into_response();
        }
    }

    if let Some(new_username) = payload.username {
        if new_username.is_empty() {
            return (StatusCode::BAD_REQUEST, "Username cannot be empty").into_response();
        }
        user.username = new_username;
        match get_user_by_username(&state, &user.username) {
            Ok(Some(existing_user)) if existing_user.id != user.id => {
                return (StatusCode::CONFLICT, "Username already exists").into_response();
            }
            Ok(_) => {}
            Err(_) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Error checking username")
                    .into_response();
            }
        }
    }

    match super::services::update_user(&state, &user) {
        Ok(_) => (StatusCode::OK, Json(user)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Error updating user").into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/api/users/me/signature",
    description = "Save the handwritten signature for the current user",
    request_body = SaveSignaturePayload,
    responses(
        (status = 200, description = "Signature saved successfully"),
        (status = 400, description = "Invalid signature format"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "User not found"),
    ),
    tag = "Users"
)]
pub async fn save_signature(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    Json(payload): Json<SaveSignaturePayload>,
) -> impl IntoResponse {
    if !payload.signature.starts_with("data:image/png;base64,") {
        return (StatusCode::BAD_REQUEST, "Invalid signature format. Expected PNG base64 data URL").into_response();
    }

    let user = match get_user_by_id(&state, &jwt_user.sub) {
        Ok(Some(user)) => user,
        Ok(None) => {
            error!("User not found for signature save: id={}", jwt_user.sub);
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        },
        Err(e) => {
            error!("Error fetching user for signature save: {:?}", e);
            return (StatusCode::NOT_FOUND, "User not found").into_response();
        },
    };

    info!("Adding signature for user {}: {} characters", user.username, payload.signature.len());
    match add_user_signature(&state, &user.id, &payload.signature) {
        Ok(sig) => {
            info!("Signature added successfully for user {} (id: {})", user.username, sig.id);
            (StatusCode::CREATED, Json(sig)).into_response()
        },
        Err(e) => {
            error!("Error adding signature for user {}: {:?}", user.username, e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error saving signature").into_response()
        },
    }
}

#[utoipa::path(
    get,
    path = "/api/users/me/signatures",
    description = "List all handwritten signatures for the current user",
    responses(
        (status = 200, description = "List of signatures", body = Vec<UserSignature>),
        (status = 401, description = "Unauthorized"),
    ),
    tag = "Users"
)]
pub async fn get_signatures(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
) -> impl IntoResponse {
    let user_id = jwt_user.sub.to_string();
    match get_user_signatures(&state, &user_id) {
        Ok(sigs) => (StatusCode::OK, Json(sigs)).into_response(),
        Err(e) => {
            error!("Error fetching signatures: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error fetching signatures").into_response()
        },
    }
}

#[utoipa::path(
    delete,
    path = "/api/users/me/signatures/{id}",
    description = "Delete a handwritten signature by id",
    params(("id" = String, Path, description = "Signature id")),
    responses(
        (status = 204, description = "Signature deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Signature not found"),
    ),
    tag = "Users"
)]
pub async fn delete_signature(
    State(state): State<GlobalState>,
    jwt_user: JwtClaims,
    axum::extract::Path(signature_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let user_id = jwt_user.sub.to_string();
    match delete_user_signature(&state, &signature_id, &user_id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Signature not found").into_response(),
        Err(e) => {
            error!("Error deleting signature: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Error deleting signature").into_response()
        },
    }
}
