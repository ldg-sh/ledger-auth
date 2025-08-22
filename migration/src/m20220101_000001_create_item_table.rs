use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Item::Table)
                    .col(
                        ColumnDef::new(Item::Id)
                            .string()
                            .not_null()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(Item::Name)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Item::CreatedAt)
                            .string()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Item::UpdatedAt)
                            .string()
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
                    .table(Item::Table)
                    .to_owned()
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Item {
    Table,
    Id,
    Name,
    CreatedAt,
    UpdatedAt,
}
