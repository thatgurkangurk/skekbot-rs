#![allow(clippy::future_not_send, clippy::derive_partial_eq_without_eq)]

use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "server")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: u64,

    #[sea_orm(default_value = true)]
    pub dad_enabled: bool,
}

impl ActiveModelBehavior for ActiveModel {}
