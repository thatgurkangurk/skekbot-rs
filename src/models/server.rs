#![allow(clippy::future_not_send, clippy::derive_partial_eq_without_eq)]

use sea_orm::prelude::*;
use serde::Serialize;

#[sea_orm::model]
#[derive(Serialize, Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "server")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,

    #[sea_orm(default_value = 0.25)]
    pub hidden_chance: f64,

    #[sea_orm(default_value = true)]
    pub dad_enabled: bool,
}

impl ActiveModelBehavior for ActiveModel {}
