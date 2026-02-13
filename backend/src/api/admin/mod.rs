mod endpoints;

use axum::Router;
use axum::routing::delete;

use crate::misc::GlobalState;

pub fn get_routes(state: GlobalState) -> Router {
    Router::new()
        .route("/users/{id}", delete(endpoints::delete_user))
        .with_state(state)
}
