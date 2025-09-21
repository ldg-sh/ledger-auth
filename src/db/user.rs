use crate::db::postgres_service::PostgresService;
use crate::{types::{error::AppError, token::TokenType, user}, utils::token::{self, encrypt, new_token}};
use chrono::Utc;
use entity::team::{Entity as Team, Model as TeamModel};
use entity::user::{ActiveModel as UserActive, Entity as User, Model as UserModel};
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;

impl PostgresService {
    pub async fn user_exists_by_email(&self, email: &str) -> Result<bool, AppError> {
        Ok(User::find().filter(entity::user::Column::Email.eq(email)).count(&self.database_connection).await? > 0)
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<UserModel, AppError> {
        Ok(User::find_by_id(*id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("User does not exist".into()))?)
    }

    pub async fn get_user_by_email(&self, email: String) -> Result<UserModel, AppError> {
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

    /// Signup: creates personal team + user in one txn.
    pub async fn create_user(&self, payload: user::DBUserCreate) -> Result<Uuid, AppError> {
        if self.user_exists_by_email(&payload.email).await? { return Err(AppError::AlreadyExists); }
        let uid = token::new_id();
        let now = Utc::now();
        let txn = self.database_connection.begin().await?;


        User::insert(UserActive {
            id: Set(uid),
            name: Set(payload.name),
            email: Set(payload.email),
            token: Set(payload.token),
            team_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        }).exec(&txn).await?;

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
        if self.user_exists_by_email(&email).await? { return Err(AppError::Db(DbErr::RecordNotUpdated)); }
        let mut am: UserActive = self.get_user_by_id(&user_id).await?.into();
        am.email = Set(email);
        am.updated_at = Set(Utc::now());
        Ok(am.update(&self.database_connection).await.map(|_| ())?)
    }

    pub async fn set_user_team(&self, user_id: Uuid, team_id: Uuid) -> Result<(), AppError> {
        // Ensure team exists
        Team::find_by_id(team_id).one(&self.database_connection).await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;

        // Update directly and verify a row was affected
        let res = entity::user::Entity::update_many()
            .col_expr(entity::user::Column::TeamId, Expr::value(team_id))
            .col_expr(entity::user::Column::UpdatedAt, Expr::value(Utc::now()))
            .filter(entity::user::Column::Id.eq(user_id))
            .exec(&self.database_connection)
            .await?;

        if res.rows_affected == 0 { return Err(AppError::Db(DbErr::RecordNotUpdated)); }
        Ok(())
    }

    pub async fn get_team_for_user(&self, user_id: Uuid) -> Result<TeamModel, AppError> {
        let u = self.get_user_by_id(&user_id).await?;
        let team_id = match u.team_id {
            Some(t) => t,
            None => {
                return Err(AppError::Db(DbErr::RecordNotFound("User doesn't have a team...".to_string())))
            },
        };
        Ok(Team::find_by_id(team_id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?)
    }

    pub async fn user_is_team_owner(&self, user_id: Uuid) -> Result<bool, AppError> {
        Ok(Team::find()
            .filter(entity::team::Column::Owner.eq(user_id))
            .count(&self.database_connection)
            .await? > 0)
    }

    pub async fn user_owns_team(&self, user_id: Uuid, team_id: Uuid) -> Result<bool, AppError> {
        Ok(Team::find()
            .filter(entity::team::Column::Id.eq(team_id))
            .filter(entity::team::Column::Owner.eq(user_id))
            .count(&self.database_connection)
            .await? > 0)
    }

    pub async fn user_can_access_team(&self, user_id: Uuid, team_id: Uuid) -> Result<bool, AppError> {
        let user = self.get_user_by_id(&user_id).await?;
        let team = Team::find_by_id(team_id)
            .one(&self.database_connection)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;
        if team.owner == user_id { return Ok(true); }
        if user.team_id != Some(team_id) { return Ok(false); }

        Ok(true)
    }

    pub async fn list_users_in_team_paginated(&self, team_id: Uuid, page: u64, per_page: u64)
        -> Result<(Vec<UserModel>, u64), AppError> {
        let finder = User::find().filter(entity::user::Column::TeamId.eq(team_id));
        let total = finder.clone().count(&self.database_connection).await?;
        let items = finder.paginate(&self.database_connection, per_page).fetch_page(page).await?;
        Ok((items, total))
    }

    /// Prevent deleting if the user is listed as team.owner.
    pub async fn delete_user_safe(&self, user_id: Uuid) -> Result<(), AppError> {
        let owning = Team::find().filter(entity::team::Column::Owner.eq(user_id)).count(&self.database_connection).await?;
        if owning > 0 { return Err(AppError::Db(DbErr::RecordNotUpdated)); }
        if let Some(u) = User::find_by_id(user_id).one(&self.database_connection).await? {
            let am: UserActive = u.into();
            Ok(am.delete(&self.database_connection).await.map(|_| ())?)
        } else { Ok(()) }
    }
}
