use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::UserId)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Users::Username)
                            .string_len(60)
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Users::Password).char_len(60).not_null())
                    .col(ColumnDef::new(Users::JoinDate).date_time().not_null())
                    .col(ColumnDef::new(Users::Level).tiny_unsigned().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BlogPosts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BlogPosts::PostId)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BlogPosts::Title).string_len(100).not_null())
                    .col(ColumnDef::new(BlogPosts::Body).text().not_null())
                    .col(ColumnDef::new(BlogPosts::AuthorId).unsigned().not_null())
                    .col(ColumnDef::new(BlogPosts::PostDate).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_author")
                            .from(BlogPosts::Table, BlogPosts::AuthorId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Comments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Comments::CommentId)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Comments::PostId).unsigned().not_null())
                    .col(ColumnDef::new(Comments::AuthorId).unsigned())
                    .col(ColumnDef::new(Comments::AnonName).string_len(60))
                    .col(ColumnDef::new(Comments::Body).text().not_null())
                    .col(ColumnDef::new(Comments::CommentDate).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_author")
                            .from(Comments::Table, Comments::AuthorId)
                            .to(Users::Table, Users::UserId)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_post")
                            .from(Comments::Table, Comments::PostId)
                            .to(BlogPosts::Table, BlogPosts::PostId)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comments::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(BlogPosts::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    UserId,
    Username,
    Password,
    JoinDate,
    Level,
}

#[derive(Iden)]
enum BlogPosts {
    Table,
    PostId,
    Title,
    Body,
    AuthorId,
    PostDate,
}

#[derive(Iden)]
enum Comments {
    Table,
    CommentId,
    PostId,
    AuthorId,
    AnonName,
    Body,
    CommentDate,
}
