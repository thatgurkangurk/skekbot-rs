#![allow(clippy::needless_for_each)]

use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use serenity::all::{ChannelId, MessageId};
use std::sync::Arc;
use tokio::sync::Mutex;

use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};

use serenity::Client;

use crate::Config;

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct MessageRequest {
    channel_id: String,
    message: String,
    reply_to_id: Option<String>,
}

#[derive(Clone)]
pub struct BotState {
    client: Arc<Mutex<Client>>,
    pub http: Arc<serenity::http::Http>,
    pub config: Config,
}

impl BotState {
    #[must_use]
    pub fn new(client: Client, config: &Config) -> Self {
        let config_clone = config.clone();
        Self {
            http: client.http.clone(),
            client: Arc::new(Mutex::new(client)),
            config: config_clone,
        }
    }

    pub async fn start(&self) {
        // We lock it internally; the caller never knows a Mutex even exists
        let mut lock = self.client.lock().await;
        if let Err(why) = lock.start().await {
            eprintln!("Bot Error: {why:?}");
        }
    }
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

#[utoipa::path(
    post,
    path = "/send-message",
    request_body = MessageRequest,
    responses(
        (status = 200, description = "message sent successfully"),
        (status = 400, description = "bad request (like invalid channel id)"),
        (status = 401, description = "unauthorized (missing token)"),
        (status = 403, description = "forbidden (invalid token)"),
        (status = 500, description = "internal server srror")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn send_message_handler(
    State(state): State<BotState>,
    headers: HeaderMap,
    Json(body): Json<MessageRequest>,
) -> impl IntoResponse {
    let password = state.config.web.password;

    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(t) if t == format!("Bearer {password}") => (),
        Some(_) => return (StatusCode::FORBIDDEN, "Forbidden").into_response(),
        None => return (StatusCode::UNAUTHORIZED, "Unauthorised").into_response(),
    }

    let channel_id: ChannelId = match body.channel_id.parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid channel ID format").into_response(),
    };

    let http = &state.http;

    let result = if let Some(reply_id_str) = body.reply_to_id {
        if let Ok(reply_id) = reply_id_str.parse::<u64>() {
            channel_id
                .send_message(
                    http,
                    serenity::builder::CreateMessage::new()
                        .content(body.message)
                        .reference_message((channel_id, MessageId::new(reply_id))),
                )
                .await
        } else {
            return (StatusCode::BAD_REQUEST, "Invalid replyToId format").into_response();
        }
    } else {
        channel_id.say(http, body.message).await
    };

    match result {
        Ok(_) => (StatusCode::OK, "Success!").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Discord Error: {e}"),
        )
            .into_response(),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(health_check, send_message_handler),
    components(schemas(MessageRequest)),
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
        .route("/send-message", post(send_message_handler))
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .with_state(state)
}

pub async fn run_web(state: BotState) {
    let web = create_web(state);

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("failed to bind tcp listener: {e}");
            return;
        }
    };

    println!("server running on http://localhost:3000/");

    if let Err(e) = axum::serve(listener, web).await {
        eprintln!("failed to start server: {e}");
    }
}
