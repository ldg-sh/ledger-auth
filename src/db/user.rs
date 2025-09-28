use crate::db::postgres_service::PostgresService;
use crate::{
    types::{error::AppError, token::TokenType, user},
    utils::token::{self, encrypt, new_token},
};
use chrono::Utc;
use entity::team::{Entity as Team, Model as TeamModel};
use entity::user::{ActiveModel as UserActive, Entity as User, Model as UserModel};
use entity::user_team::{ActiveModel as UserTeamActive, Entity as UserTeam};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, JoinType, PaginatorTrait, QueryFilter,
    QuerySelect, Set, SqlErr, TransactionTrait,
};
use uuid::Uuid;

impl PostgresService {
    pub async fn user_exists_by_email(&self, email: &str) -> Result<bool, AppError> {
        Ok(User::find()
            .filter(entity::user::Column::Email.eq(email))
            .count(&self.database_connection)
            .await?
            > 0)
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<UserModel, AppError> {
        Ok(User::find_by_id(*id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("User does not exist".into()))?)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<UserModel, AppError> {
        Ok(User::find()
            .filter(entity::user::Column::Email.eq(email))
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("User does not exist".into()))?)
    }

    pub async fn get_user_token(&self, id: Uuid) -> Result<String, AppError> {
        Ok(self.get_user_by_id(&id).await?.token)
    }

    pub async fn regenerate_user_token(&self, user_id: &Uuid) -> Result<String, AppError> {
        let user = self.get_user_by_id(user_id).await?;
        let token = new_token(TokenType::User);
        let encrypted = encrypt(&token).map_err(|_| DbErr::RecordNotUpdated)?;
        let mut am: UserActive = user.into();
        am.token = Set(encrypted);
        am.updated_at = Set(Utc::now());
        am.update(&self.database_connection).await?;
        Ok(token)
    }

    /// Signup: create user.
    pub async fn create_user(&self, payload: user::DBUserCreate) -> Result<Uuid, AppError> {
        if self.user_exists_by_email(&payload.email).await? {
            return Err(AppError::AlreadyExists);
        }
        let uid = token::new_id();
        let now = Utc::now();
        let txn = self.database_connection.begin().await?;

        User::insert(UserActive {
            id: Set(uid),
            name: Set(payload.name),
            email: Set(payload.email),
            token: Set(payload.token),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(&txn)
        .await?;

        txn.commit().await?;
        Ok(uid)
    }

    pub async fn update_user_name(&self, user_id: Uuid, name: String) -> Result<(), AppError> {
        let mut am: UserActive = self.get_user_by_id(&user_id).await?.into();
        am.name = Set(name);
        am.updated_at = Set(Utc::now());
        Ok(am.update(&self.database_connection).await.map(|_| ())?)
    }

    pub async fn update_user_email(&self, user_id: Uuid, email: String) -> Result<(), AppError> {
        if self.user_exists_by_email(&email).await? {
            return Err(AppError::Db(DbErr::RecordNotUpdated));
        }
        let mut am: UserActive = self.get_user_by_id(&user_id).await?.into();
        am.email = Set(email);
        am.updated_at = Set(Utc::now());
        Ok(am.update(&self.database_connection).await.map(|_| ())?)
    }

    /// Add the user to a team. Avoid duplicates
    pub async fn set_user_team(&self, user_id: Uuid, team_id: Uuid) -> Result<(), AppError> {
        // Surface a 404 if the user does not exist
        self.get_user_by_id(&user_id).await?;

        // Ensure team exists (separate from membership insert for clearer errors)
        Team::find_by_id(team_id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;

        let now = Utc::now();
        match UserTeam::insert(UserTeamActive {
            user_id: Set(user_id),
            team_id: Set(team_id),
            created_at: Set(now),
        })
        .exec(&self.database_connection)
        .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                if matches!(err.sql_err(), Some(SqlErr::UniqueConstraintViolation(_))) {
                    Err(AppError::AlreadyExists)
                } else {
                    Err(err.into())
                }
            }
        }
    }

    pub async fn get_team_for_user(&self, user_id: Uuid) -> Result<TeamModel, AppError> {
        // Preserve behavior: return "a" team for user.
        // Prefer owned team if any; else first membership; else legacy column.
        if let Some(t) = Team::find()
            .filter(entity::team::Column::Owner.eq(user_id))
            .one(&self.database_connection)
            .await?
        {
            return Ok(t);
        }

        if let Some(row) = UserTeam::find()
            .filter(entity::user_team::Column::UserId.eq(user_id))
            .one(&self.database_connection)
            .await?
        {
            let t = Team::find_by_id(row.team_id)
                .one(&self.database_connection)
                .await?
                .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;
            return Ok(t);
        }

        Err(AppError::Db(DbErr::RecordNotFound(
            "User doesn't have a team...".to_string(),
        )))
    }

    /// Is the user (any) team owner.
    pub async fn user_is_team_owner(&self, user_id: Uuid) -> Result<bool, AppError> {
        Ok(Team::find()
            .filter(entity::team::Column::Owner.eq(user_id))
            .count(&self.database_connection)
            .await?
            > 0)
    }

    pub async fn user_owns_team(&self, user_id: Uuid, team_id: Uuid) -> Result<bool, AppError> {
        Ok(Team::find()
            .filter(entity::team::Column::Id.eq(team_id))
            .filter(entity::team::Column::Owner.eq(user_id))
            .count(&self.database_connection)
            .await?
            > 0)
    }

    pub async fn user_can_access_team(
        &self,
        user_id: Uuid,
        team_id: Uuid,
    ) -> Result<bool, AppError> {
        let team = Team::find_by_id(team_id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;
        if team.owner == user_id {
            return Ok(true);
        }
        let exists = UserTeam::find()
            .filter(entity::user_team::Column::UserId.eq(user_id))
            .filter(entity::user_team::Column::TeamId.eq(team_id))
            .count(&self.database_connection)
            .await?
            > 0;
        Ok(exists)
    }

    pub async fn list_users_in_team_paginated(
        &self,
        team_id: Uuid,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<UserModel>, u64), AppError> {
        let finder = User::find()
            .join(
                JoinType::InnerJoin,
                UserTeam::belongs_to(User)
                    .from(entity::user_team::Column::UserId)
                    .to(entity::user::Column::Id)
                    .into(),
            )
            .filter(entity::user_team::Column::TeamId.eq(team_id));
        let total = finder.clone().count(&self.database_connection).await?;
        let items = finder
            .paginate(&self.database_connection, per_page)
            .fetch_page(page)
            .await?;
        Ok((items, total))
    }

    /// Prevent deleting if the user is listed as team.owner.
    pub async fn delete_user_safe(&self, user_id: Uuid) -> Result<(), AppError> {
        let owning = Team::find()
            .filter(entity::team::Column::Owner.eq(user_id))
            .count(&self.database_connection)
            .await?;
        if owning > 0 {
            return Err(AppError::Db(DbErr::RecordNotUpdated));
        }
        if let Some(u) = User::find_by_id(user_id)
            .one(&self.database_connection)
            .await?
        {
            let am: UserActive = u.into();
            Ok(am.delete(&self.database_connection).await.map(|_| ())?)
        } else {
            Ok(())
        }
    }
}
