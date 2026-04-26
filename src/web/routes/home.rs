use crate::{
    consts,
    web::{AppError, BotState},
};
use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a> {
    version: &'a str,
}

pub async fn home(State(state): State<BotState>) -> Result<impl IntoResponse, AppError> {
    let template = HomeTemplate {
        version: consts::VERSION,
    };
    Ok(Html(template.render()?))
}
