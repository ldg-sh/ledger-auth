use sea_orm_migration::{prelude::*, sea_query::TableForeignKey};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[allow(dead_code)]
#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Email,
    TeamId,
    Token,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Team {
    Table,
    Id,
    Name,
    Owner,
    CreatedAt,
    UpdatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // UUID helper (optional)
        m.get_connection()
            .execute_unprepared(r#"CREATE EXTENSION IF NOT EXISTS "pgcrypto";"#)
            .await?;

        // teams
        m.create_table(
            Table::create()
                .table(Team::Table)
                .col(ColumnDef::new(Team::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Team::Name).string().not_null())
                .col(ColumnDef::new(Team::Owner).uuid().not_null())
                .col(ColumnDef::new(Team::CreatedAt).timestamp_with_time_zone().not_null())
                .col(ColumnDef::new(Team::UpdatedAt).timestamp_with_time_zone().not_null())
                .to_owned(),
        ).await?;

        m.create_index(
            Index::create()
                .name("uk_team_name_owner")
                .table(Team::Table)
                .col(Team::Name)
                .col(Team::Owner)
                .unique()
                .to_owned(),
        ).await?;

        // users.team_id (nullable during backfill)
        m.alter_table(
            Table::alter()
                .table(User::Table)
                .add_column(ColumnDef::new(User::TeamId).uuid().null())
                .to_owned(),
        ).await?;

        // bootstrap personal teams
        m.get_connection().execute_unprepared(
            r#"
            INSERT INTO team (id,name,owner,created_at,updated_at)
            SELECT gen_random_uuid(),
                   COALESCE(NULLIF(u.name, ''), 'User') || ' Team',
                   u.id,
                   NOW(), NOW()
            FROM "user" u
            WHERE NOT EXISTS (SELECT 1 FROM team t WHERE t.owner = u.id);
            "#,
        ).await?;

        // backfill users.team_id
        m.get_connection().execute_unprepared(
            r#"
            UPDATE "user" u
            SET team_id = t.id
            FROM team t
            WHERE t.owner = u.id AND u.team_id IS NULL;
            "#,
        ).await?;

        // enforce NOT NULL
        m.alter_table(
            Table::alter()
                .table(User::Table)
                .modify_column(ColumnDef::new(User::TeamId).uuid().not_null())
                .to_owned(),
        ).await?;

        // FK + index
        m.alter_table(
            Table::alter()
                .table(User::Table)
                .add_foreign_key(
                    TableForeignKey::new()
                        .name("fk_user_team")
                        .from_tbl(User::Table)
                        .from_col(User::TeamId)
                        .to_tbl(Team::Table)
                        .to_col(Team::Id)
                        .on_delete(ForeignKeyAction::Restrict)
                        .on_update(ForeignKeyAction::Cascade),
                )
                .to_owned(),
        ).await?;

        m.create_index(
            Index::create()
                .name("idx_user_team_id")
                .table(User::Table)
                .col(User::TeamId)
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.alter_table(
            Table::alter()
                .table(User::Table)
                .drop_foreign_key(Alias::new("fk_user_team"))
                .to_owned(),
        ).await?;
        m.drop_index(Index::drop().name("idx_user_team_id").table(User::Table).to_owned()).await?;

        m.alter_table(
            Table::alter()
                .table(User::Table)
                .drop_column(User::TeamId)
                .to_owned(),
        ).await?;

        m.drop_index(Index::drop().name("uk_team_name_owner").table(Team::Table).to_owned()).await?;
        m.drop_table(Table::drop().table(Team::Table).to_owned()).await?;
        Ok(())
    }
}
