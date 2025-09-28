use crate::db::postgres_service::PostgresService;
use crate::types::error::AppError;
use chrono::Utc;
use entity::team::{ActiveModel as TeamActive, Entity as Team, Model as TeamModel};
use entity::user::{Entity as User, Model as UserModel};
use entity::user_team::Entity as UserTeam;
use sea_orm::JoinType;
use sea_orm::QuerySelect;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set,
    TransactionTrait,
};
use uuid::Uuid;

impl PostgresService {
    pub async fn team_exists_by_name_owner(
        &self,
        owner: Uuid,
        name: &str,
    ) -> Result<bool, AppError> {
        Ok(Team::find()
            .filter(entity::team::Column::Owner.eq(owner))
            .filter(entity::team::Column::Name.eq(name))
            .count(&self.database_connection)
            .await?
            > 0)
    }

    pub async fn create_team(&self, owner: Uuid, name: String) -> Result<Uuid, AppError> {
        if self.team_exists_by_name_owner(owner, &name).await? {
            return Err(AppError::AlreadyExists);
        }

        let team_id = Uuid::new_v4();
        let now = Utc::now();

        Team::insert(TeamActive {
            id: Set(team_id),
            name: Set(name),
            owner: Set(owner),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(&self.database_connection)
        .await?;

        Ok(team_id)
    }

    pub async fn get_team(&self, id: Uuid) -> Result<TeamModel, AppError> {
        Ok(Team::find_by_id(id)
            .one(&self.database_connection)
            .await?
            .ok_or(DbErr::RecordNotFound("Team not found".to_string()))?)
    }

    pub async fn list_users_in_team(&self, team_id: Uuid) -> Result<Vec<Uuid>, AppError> {
        let users = UserTeam::find()
            .filter(entity::user_team::Column::TeamId.eq(team_id))
            .all(&self.database_connection)
            .await?;

        Ok(users.iter().map(|i| i.user_id).collect())
    }

    pub async fn list_teams_paginated(
        &self,
        owner: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<TeamModel>, u64), AppError> {
        let finder = Team::find().filter(entity::team::Column::Owner.eq(owner));

        let total = finder.clone().count(&self.database_connection).await?;
        let items = finder
            .paginate(&self.database_connection, per_page)
            .fetch_page(page)
            .await?;

        Ok((items, total))
    }

    pub async fn rename_team(&self, team_id: Uuid, new_name: String) -> Result<(), AppError> {
        let t = Team::find_by_id(team_id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;

        if self.team_exists_by_name_owner(t.owner, &new_name).await? {
            return Err(AppError::Db(DbErr::RecordNotUpdated));
        }

        let mut am: TeamActive = t.into();
        am.name = Set(new_name);
        am.updated_at = Set(Utc::now());

        Ok(am.update(&self.database_connection).await.map(|_| ())?)
    }

    pub async fn transfer_team_ownership(
        &self,
        team_id: Uuid,
        new_owner: Uuid,
    ) -> Result<(), AppError> {
        let mut am: TeamActive = Team::find_by_id(team_id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?
            .into();

        am.owner = Set(new_owner);
        am.updated_at = Set(Utc::now());
        Ok(am.update(&self.database_connection).await.map(|_| ())?)
    }

    /// Delete only if no users belong (keeps invariant).
    pub async fn delete_team_if_empty(&self, team_id: Uuid) -> Result<(), AppError> {
        let txn = self.database_connection.begin().await?;

        let cnt = UserTeam::find()
            .filter(entity::user_team::Column::TeamId.eq(team_id))
            .count(&txn)
            .await?;
        if cnt > 0 {
            txn.rollback().await?;
            return Err(AppError::Db(DbErr::RecordNotUpdated));
        }

        if let Some(t) = Team::find_by_id(team_id).one(&txn).await? {
            let am: TeamActive = t.into();
            am.delete(&txn).await?;
            txn.commit().await?;
            Ok(())
        } else {
            txn.rollback().await?;
            Err(AppError::Db(DbErr::RecordNotFound("Team not found".into())))
        }
    }
}
