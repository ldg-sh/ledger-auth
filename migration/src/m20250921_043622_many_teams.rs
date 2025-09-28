use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    TeamId,
}

#[derive(DeriveIden)]
enum Team {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum UserTeam {
    Table,
    UserId,
    TeamId,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Create user_team join table
        m.create_table(
            Table::create()
                .table(UserTeam::Table)
                .if_not_exists()
                .col(ColumnDef::new(UserTeam::UserId).uuid().not_null())
                .col(ColumnDef::new(UserTeam::TeamId).uuid().not_null())
                .col(ColumnDef::new(UserTeam::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .primary_key(
                    Index::create()
                        .name("pk_user_team")
                        .col(UserTeam::UserId)
                        .col(UserTeam::TeamId)
                )
                .to_owned(),
        ).await?;

        // FKs
        m.alter_table(
            Table::alter()
                .table(UserTeam::Table)
                .add_foreign_key(
                    TableForeignKey::new()
                        .name("fk_user_team_user")
                        .from_tbl(UserTeam::Table)
                        .from_col(UserTeam::UserId)
                        .to_tbl(User::Table)
                        .to_col(User::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .add_foreign_key(
                    TableForeignKey::new()
                        .name("fk_user_team_team")
                        .from_tbl(UserTeam::Table)
                        .from_col(UserTeam::TeamId)
                        .to_tbl(Team::Table)
                        .to_col(Team::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .to_owned(),
        ).await?;

        m.create_index(
            Index::create()
                .name("idx_user_team_user")
                .table(UserTeam::Table)
                .col(UserTeam::UserId)
                .to_owned(),
        ).await?;

        m.create_index(
            Index::create()
                .name("idx_user_team_team")
                .table(UserTeam::Table)
                .col(UserTeam::TeamId)
                .to_owned(),
        ).await?;

        // Backfill: for existing single-team assignments, mirror into user_team
        // Safe to run multiple times due to PK(user_id, team_id)
        m.get_connection().execute_unprepared(
            r#"
            INSERT INTO user_team (user_id, team_id, created_at)
            SELECT u.id, u.team_id, NOW()
            FROM "user" u
            WHERE u.team_id IS NOT NULL
            ON CONFLICT (user_id, team_id) DO NOTHING;
            "#,
        ).await?;

        // Drop legacy column user.team_id now that memberships are backfilled
        m.alter_table(
            Table::alter()
                .table(User::Table)
                .drop_column(User::TeamId)
                .to_owned(),
        ).await?;

        Ok(())
    }

    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        // Recreate legacy column (nullable) on rollback; FKs are not restored here
        m.alter_table(
            Table::alter()
                .table(User::Table)
                .add_column(ColumnDef::new(User::TeamId).uuid().null())
                .to_owned(),
        ).await?;

        // Best-effort restore: set user.team_id to any membership (arbitrary)
        m.get_connection().execute_unprepared(
            r#"
            UPDATE "user" u
            SET team_id = ut.team_id
            FROM (
                SELECT DISTINCT ON (user_id) user_id, team_id
                FROM user_team
                ORDER BY user_id, created_at ASC
            ) ut
            WHERE ut.user_id = u.id;
            "#,
        ).await?;

        // Drop indexes implicitly with table
        m.drop_table(Table::drop().table(UserTeam::Table).to_owned()).await?;
        Ok(())
    }
}
