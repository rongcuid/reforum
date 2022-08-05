//! SeaORM Entity. Generated by sea-orm-codegen 0.9.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub password: Option<String>,
    pub signature: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub last_seen_at: Option<String>,
    pub last_post_at: Option<String>,
    pub muted_until: Option<String>,
    pub banned_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::moderators::Entity")]
    Moderators,
    #[sea_orm(has_many = "super::past_moderators::Entity")]
    PastModerators,
    #[sea_orm(has_many = "super::topics::Entity")]
    Topics,
    #[sea_orm(has_many = "super::posts::Entity")]
    Posts,
    #[sea_orm(has_many = "super::replies::Entity")]
    Replies,
    #[sea_orm(has_many = "super::sessions::Entity")]
    Sessions,
}

impl Related<super::moderators::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Moderators.def()
    }
}

impl Related<super::past_moderators::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PastModerators.def()
    }
}

impl Related<super::topics::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Topics.def()
    }
}

impl Related<super::posts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Posts.def()
    }
}

impl Related<super::replies::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Replies.def()
    }
}

impl Related<super::sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}