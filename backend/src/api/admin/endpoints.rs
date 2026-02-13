use axum::{
    extract::{State, Path},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use tracing::error;

use crate::{
    api::users::delete_user_account,
    misc::GlobalState,
};

const X_ADMIN_KEY: &str = "x-admin-key";

/// DELETE /api/admin/users/:id — supprime un utilisateur par son id (nécessite X-Admin-Key).
pub async fn delete_user(
    State(state): State<GlobalState>,
    Path(user_id): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let expected = match &state.admin_key {
        Some(k) => k.as_str(),
        None => {
            return (
                StatusCode::NOT_IMPLEMENTED,
                "Admin key not configured (ADMIN_KEY)",
            )
                .into_response()
        }
    };
    let provided = headers
        .get(X_ADMIN_KEY)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let provided = match provided {
        Some(p) => p,
        None => {
            return (
                StatusCode::FORBIDDEN,
                "Missing or invalid X-Admin-Key header",
            )
                .into_response()
        }
    };
    if provided != expected {
        return (
            StatusCode::FORBIDDEN,
            "Invalid X-Admin-Key",
        )
            .into_response();
    }

    match delete_user_account(&state, &user_id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => {
            error!("Error deleting user {}: {:?}", user_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error deleting user",
            )
                .into_response()
        }
    }
}
