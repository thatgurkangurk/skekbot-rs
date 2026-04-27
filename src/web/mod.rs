#![allow(clippy::needless_for_each)]

mod routes;
mod state;

use askama::Template;
use axum::{
    Json, Router,
    http::Uri,
    response::{Html, IntoResponse, Response},
    routing::{get, post, put},
};
use reqwest::{StatusCode, header};
use serde::Serialize;
use serde_json::json;
use tower_http::services::ServeDir;
use tracing::{error, info};
use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};

use routes::activity::{SetActivityRequest, set_activity_handler};

use crate::web::routes::message::MessageRequest;

pub use state::BotState;

use rust_embed::RustEmbed;

#[derive(RustEmbed, Clone)]
#[folder = "assets/"]
struct Assets;

#[derive(Debug, displaydoc::Display, thiserror::Error)]
enum AppError {
    /// could not render template
    Render(#[from] askama::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct Tmpl {}

        let status = match &self {
            Self::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let tmpl = Tmpl {};
        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "server is healthy", body = serde_json::Value)
    )
)]
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
    }))
}

#[derive(OpenApi)]
#[openapi(
    paths(health_check, routes::message::send_message_handler, routes::activity::set_activity_handler),
    components(schemas(MessageRequest, SetActivityRequest, ApiResponse)),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            );
        }
    }
}

fn create_web(state: BotState) -> Router {
    let router = Router::new()
        .route("/", get(routes::home::home))
        .route("/health", get(health_check))
        .route("/send-message", post(routes::message::send_message_handler))
        .route("/activity", put(set_activity_handler));

    let router = if cfg!(debug_assertions) {
        router.fallback_service(ServeDir::new("assets"))
    } else {
        router.fallback(static_handler)
    };

    router
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .with_state(state)
}
pub async fn run_web(state: BotState) {
    let web = create_web(state);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            error!("failed to bind tcp listener: {e}");
            return;
        }
    };

    info!("server running on http://localhost:3000/");

    if let Err(e) = axum::serve(listener, web).await {
        error!("failed to start server: {e}");
    }
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
