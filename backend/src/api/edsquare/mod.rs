pub mod endpoints;
pub mod models;
pub mod services;

use axum::Router;
use crate::misc::GlobalState;

pub fn get_routes(state: GlobalState) -> Router {
    Router::new()
        .route("/validate", axum::routing::post(endpoints::validate_edsquare))
        .route("/validate-multi", axum::routing::post(endpoints::validate_edsquare_multi))
        .route("/cookies", axum::routing::post(endpoints::save_edsquare_cookies_endpoint))
        .route("/login", axum::routing::post(endpoints::login_edsquare_endpoint))
        .route("/status", axum::routing::get(endpoints::get_edsquare_status))
        .route("/eligible-users", axum::routing::get(endpoints::get_edsquare_eligible_users))
        .route("/planning-events", axum::routing::get(endpoints::get_planning_events))
        .with_state(state)
}
