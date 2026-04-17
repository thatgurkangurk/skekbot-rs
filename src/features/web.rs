#![allow(clippy::needless_for_each)]

use axum::{
    Json, Router,
    extract::State,
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post, put},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serenity::all::{ActivityData, ActivityType, ChannelId, MessageId, ShardManager};
use std::sync::Arc;
use tracing::{error, info};

use utoipa::{
    Modify, OpenApi, ToSchema,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_scalar::{Scalar, Servable};

use serenity::Client;

use crate::Config;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum ActivityTypeProxy {
    Playing,
    Streaming,
    Listening,
    Watching,
    Competing,
}

impl From<ActivityTypeProxy> for ActivityType {
    fn from(proxy: ActivityTypeProxy) -> Self {
        match proxy {
            ActivityTypeProxy::Playing => Self::Playing,
            ActivityTypeProxy::Streaming => Self::Streaming,
            ActivityTypeProxy::Listening => Self::Listening,
            ActivityTypeProxy::Watching => Self::Watching,
            ActivityTypeProxy::Competing => Self::Competing,
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct MessageRequest {
    channel_id: String,
    message: String,
    reply_to_id: Option<String>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct SetActivityRequest {
    activity_type: ActivityTypeProxy,
    text: String,
}

#[derive(Clone)]
pub struct BotState {
    pub shard_manager: Arc<ShardManager>,
    pub http: Arc<serenity::http::Http>,
    pub config: Config,
}

#[derive(Serialize, ToSchema)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

impl BotState {
    #[must_use]
    pub fn new(client: &Client, config: &Config) -> Self {
        let config_clone = config.clone();
        Self {
            shard_manager: client.shard_manager.clone(),
            http: client.http.clone(),
            config: config_clone,
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
    put,
    path = "/activity",
    request_body = SetActivityRequest,
    responses(
        (status = 200, description = "status set successfully", body = ApiResponse),
        (status = 401, description = "unauthorised (missing token)", body = ApiResponse),
        (status = 403, description = "forbidden (invalid token)", body = ApiResponse),
        (status = 500, description = "internal server error", body = ApiResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn set_activity_handler(
    State(state): State<BotState>,
    headers: HeaderMap,
    Json(body): Json<SetActivityRequest>,
) -> impl IntoResponse {
    let expected_auth = format!("Bearer {}", state.config.web.password);
    match headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        Some(token) if token == expected_auth => (),
        Some(_) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse {
                    success: false,
                    message: "Forbidden".to_string(),
                }),
            )
                .into_response();
        }
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    message: "Unauthorised".to_string(),
                }),
            )
                .into_response();
        }
    }

    let activity_type: ActivityType = body.activity_type.into();
    let activity = match activity_type {
        ActivityType::Listening => ActivityData::listening(&body.text),
        ActivityType::Watching => ActivityData::watching(&body.text),
        ActivityType::Competing => ActivityData::competing(&body.text),
        ActivityType::Custom => ActivityData::custom(&body.text),
        ActivityType::Streaming => ActivityData::streaming(&body.text, "https://twitch.tv/discord")
            .unwrap_or_else(|_| ActivityData::playing(&body.text)),
        _ => ActivityData::playing(&body.text),
    };

    let messengers: Vec<_> = {
        let runners = state.shard_manager.runners.lock().await;
        runners
            .values()
            .map(|runner| runner.runner_tx.clone())
            .collect()
    };

    for messenger in messengers {
        messenger.set_activity(Some(activity.clone()));
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Status updated successfully!".to_string(),
        }),
    )
        .into_response()
}

#[utoipa::path(
    post,
    path = "/send-message",
    request_body = MessageRequest,
    responses(
        (status = 200, description = "message sent successfully", body = ApiResponse),
        (status = 400, description = "bad request (like invalid channel id)", body = ApiResponse),
        (status = 401, description = "unauthorised (missing token)", body = ApiResponse),
        (status = 403, description = "forbidden (invalid token)", body = ApiResponse),
        (status = 500, description = "internal server error", body = ApiResponse)
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
        Some(_) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse {
                    success: false,
                    message: "Forbidden".to_string(),
                }),
            )
                .into_response();
        }
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponse {
                    success: false,
                    message: "Unauthorised".to_string(),
                }),
            )
                .into_response();
        }
    }

    let channel_id: ChannelId = match body.channel_id.parse::<u64>() {
        Ok(id) => ChannelId::new(id),
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: "Invalid channel ID format".to_string(),
                }),
            )
                .into_response();
        }
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
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    success: false,
                    message: "Invalid replyToId format".to_string(),
                }),
            )
                .into_response();
        }
    } else {
        channel_id.say(http, body.message).await
    };

    match result {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse {
                success: true,
                message: "Success!".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                message: format!("Discord Error: {e}"),
            }),
        )
            .into_response(),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(health_check, send_message_handler, set_activity_handler),
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
        .route("/send-message", post(send_message_handler))
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
