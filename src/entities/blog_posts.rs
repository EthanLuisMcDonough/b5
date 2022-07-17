//! SeaORM Entity. Generated by sea-orm-codegen 0.9.0

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "blog_posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub post_id: u32,
    pub title: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
    pub author_id: u32,
    pub post_date: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::AuthorId",
        to = "super::users::Column::UserId",
        on_update = "Cascade",
        on_delete = "NoAction"
    )]
    Users,
    #[sea_orm(has_many = "super::comments::Entity")]
    Comments,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::comments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
