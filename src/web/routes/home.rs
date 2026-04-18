use axum::{extract::State, response::{Html, IntoResponse}};
use crate::web::{BotState, AppError};
use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    version: &'a str,
}

pub async fn home(
    State(state): State<BotState>,
) -> Result<impl IntoResponse, AppError> {
    let template = HomeTemplate { version: "hi" };
    Ok(Html(template.render()?))
}