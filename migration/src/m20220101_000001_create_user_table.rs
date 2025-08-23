use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .col(
                        ColumnDef::new(User::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(User::Name)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(User::Email)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(User::Token)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                    )
                    .to_owned()
            )
            .await?;
        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(User::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Email,
    Token,
    CreatedAt,
    UpdatedAt,
}
