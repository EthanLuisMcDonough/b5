use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(BlogPosts::Table)
                    .add_column(ColumnDef::new(BlogPosts::Description).string_len(200))
                    .add_column(ColumnDef::new(BlogPosts::LastUpdated).date_time())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(BlogPosts::Table)
                    .drop_column(BlogPosts::Description)
                    .drop_column(BlogPosts::LastUpdated)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum BlogPosts {
    Table,
    Description,
    LastUpdated,
}
