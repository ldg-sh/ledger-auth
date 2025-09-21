use crate::db::postgres_service::PostgresService;
use crate::types::error::AppError;
use chrono::Utc;
use entity::team::{ActiveModel as TeamActive, Entity as Team, Model as TeamModel};
use entity::user::{ActiveModel as UserActive, Entity as User, Model as UserModel};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;

impl PostgresService {
    pub async fn team_exists_by_name_owner(&self, owner: Uuid, name: &str) -> Result<bool, AppError> {
        Ok(Team::find()
            .filter(entity::team::Column::Owner.eq(owner))
            .filter(entity::team::Column::Name.eq(name))
            .count(&self.database_connection).await? > 0)
    }

    pub async fn create_team(&self, owner: Uuid, name: String) -> Result<Uuid, AppError> {
        if self.team_exists_by_name_owner(owner, &name).await? { return Err(AppError::AlreadyExists); }
        let tid = Uuid::new_v4();
        let now = Utc::now();
        Team::insert(TeamActive {
            id: Set(tid),
            name: Set(name),
            owner: Set(owner),
            created_at: Set(now),
            updated_at: Set(now),
        }).exec(&self.database_connection).await?;
        Ok(tid)
    }

    pub async fn get_team(&self, id: Uuid) -> Result<TeamModel, AppError> {
        Ok(Team::find_by_id(id).one(&self.database_connection).await?.ok_or(DbErr::RecordNotFound("Team not found".to_string()))?)
    }

    pub async fn list_teams_for_owner(&self, owner: Uuid) -> Result<Vec<TeamModel>, AppError> {
        Ok(Team::find().filter(entity::team::Column::Owner.eq(owner)).all(&self.database_connection).await?)
    }

    pub async fn list_users_in_team(&self, team_id: Uuid) -> Result<Vec<UserModel>, AppError> {
        Ok(User::find().filter(entity::user::Column::TeamId.eq(team_id)).all(&self.database_connection).await?)
    }

    pub async fn list_teams_paginated(&self, owner: Uuid, page: u64, per_page: u64)
        -> Result<(Vec<TeamModel>, u64), AppError> {
        let finder = Team::find().filter(entity::team::Column::Owner.eq(owner));
        let total = finder.clone().count(&self.database_connection).await?;
        let items = finder.paginate(&self.database_connection, per_page).fetch_page(page).await?;
        Ok((items, total))
    }

    pub async fn rename_team(&self, team_id: Uuid, new_name: String) -> Result<(), AppError> {
        // optional: enforce unique (owner,new_name) by read owner first
        let t = Team::find_by_id(team_id).one(&self.database_connection).await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;
        if self.team_exists_by_name_owner(t.owner, &new_name).await? { return Err(AppError::Db(DbErr::RecordNotUpdated)); }
        let mut am: TeamActive = t.into();
        am.name = Set(new_name);
        am.updated_at = Set(Utc::now());
        Ok(am.update(&self.database_connection).await.map(|_| ())?)
    }

    pub async fn transfer_team_ownership(&self, team_id: Uuid, new_owner: Uuid) -> Result<(), AppError> {
        let mut am: TeamActive = Team::find_by_id(team_id).one(&self.database_connection).await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?.into();
        am.owner = Set(new_owner);
        am.updated_at = Set(Utc::now());
        Ok(am.update(&self.database_connection).await.map(|_| ())?)
    }

    /// Delete only if no users belong (keeps invariant).
    pub async fn delete_team_if_empty(&self, team_id: Uuid) -> Result<(), AppError> {
        let txn = self.database_connection.begin().await?;
        let cnt = User::find().filter(entity::user::Column::TeamId.eq(team_id)).count(&txn).await?;
        if cnt > 0 { txn.rollback().await?; return Err(AppError::Db(DbErr::RecordNotUpdated)); }
        if let Some(t) = Team::find_by_id(team_id).one(&txn).await? {
            let am: TeamActive = t.into();
            am.delete(&txn).await?;
            txn.commit().await?;
            Ok(())
        } else { txn.rollback().await?; Err(AppError::Db(DbErr::RecordNotFound("Team not found".into()))) }
    }

    /// Merge: reassign members to dest, then delete src.
    pub async fn delete_team_migrate_members(&self, src_team: Uuid, dest_team: Uuid) -> Result<(), AppError> {
        if src_team == dest_team { return Ok(()); }
        // ensure dest exists
        Team::find_by_id(dest_team).one(&self.database_connection).await?
            .ok_or_else(|| DbErr::RecordNotFound("Destination team not found".into()))?;

        let txn = self.database_connection.begin().await?;
        let users = User::find().filter(entity::user::Column::TeamId.eq(src_team)).all(&txn).await?;
        for u in users {
            let mut am: UserActive = u.into();
            am.team_id = Set(Some(dest_team));
            am.updated_at = Set(Utc::now());
            am.update(&txn).await?;
        }
        if let Some(src) = Team::find_by_id(src_team).one(&txn).await? {
            let am: TeamActive = src.into();
            am.delete(&txn).await?;
        } else { txn.rollback().await?; return Err(AppError::Db(DbErr::RecordNotFound("Source team not found".into()))); }
        txn.commit().await?;
        Ok(())
    }
}
