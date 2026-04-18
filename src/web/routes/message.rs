#![allow(clippy::needless_for_each)]

use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use reqwest::StatusCode;
use serde::Deserialize;
use serenity::all::{ChannelId, MessageId};

use utoipa::ToSchema;

use crate::web::{ApiResponse, BotState};

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MessageRequest {
    channel_id: String,
    message: String,
    reply_to_id: Option<String>,
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
pub async fn send_message_handler(
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
