use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serenity::all::{ActivityData, ActivityType};

use utoipa::ToSchema;

use crate::web::{ApiResponse, BotState};

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
pub struct SetActivityRequest {
    activity_type: ActivityTypeProxy,
    text: String,
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
pub async fn set_activity_handler(
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
