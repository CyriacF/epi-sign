use axum::{Router, middleware::from_fn};
use tower_cookies::CookieManagerLayer;
use tower_http::{
    cors,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{self, swagger::Swagger},
    misc::GlobalState,
};

pub fn get_router() -> Router {
    let state = GlobalState::new();

    // Routes admin (clé X-Admin-Key, pas de JWT) — à merger avant les routes protégées
    let admin_routes = Router::new()
        .nest("/api/admin", crate::api::admin::get_routes(state.clone()));

    let api_routes = Router::new()
        .nest("/api/sign", crate::api::sign::get_routes(state.clone()))
        .nest("/api/users", crate::api::users::get_routes(state.clone()))
        .nest("/api/edsquare", crate::api::edsquare::get_routes(state.clone()))
        .layer(from_fn(api::auth::auth_middleware));

    Router::new()
        .merge(admin_routes)
        .merge(api_routes)
        .nest(
            "/api/auth",
            crate::api::auth::get_no_auth_routes(state.clone()),
        )
        .layer(CookieManagerLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .merge(SwaggerUi::new("/api/docs").url("/api/docs/openapi.json", Swagger::openapi()))
        .layer(
            cors::CorsLayer::new()
                .allow_origin([axum::http::HeaderValue::from_static(
                    "http://localhost:5173",
                )]) // or use .allow_origin(AllowOrigin::same_origin()) if you want only same-origin
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PATCH,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::ACCEPT,
                    axum::http::header::HeaderName::from_static("x-admin-key"),
                ])
                .allow_credentials(true),
        )
}

pub async fn start_server(router: Router) {
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(listener) => listener,
        Err(e) => {
            tracing::error!("Failed to bind to port 3000: {}", e);
            std::process::exit(1);
        }
    };

    tracing::info!("Server listening on: {}", listener.local_addr().unwrap());
    match axum::serve(listener, router).await {
        Ok(_) => tracing::info!("Server started  on port 3000"),
        Err(e) => tracing::error!("Failed to start server: {}", e),
    };
}
