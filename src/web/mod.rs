#![allow(clippy::needless_for_each)]

mod routes;
mod state;

use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, post, put},
};
use serde::Serialize;
use serde_json::json;
use tracing::{error, info};

use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};

use routes::activity::{SetActivityRequest, set_activity_handler};

use crate::web::routes::message::MessageRequest;

pub use state::BotState;

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
    Router::new()
        .route("/health", get(health_check))
        .route("/send-message", post(routes::message::send_message_handler))
        .route("/activity", put(set_activity_handler))
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
