use chrono::Utc;
use uuid::Uuid;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionTrait};
use crate::{types::{error::AppError, token::TokenType, user}, utils::token::{self, encrypt, new_token}};
use crate::db::postgres_service::PostgresService;
use entity::user::{ActiveModel as UserActive, Entity as User, Model as UserModel};
use entity::team::{ActiveModel as TeamActive, Entity as Team, Model as TeamModel};

impl PostgresService {
    pub async fn user_exists_by_email(&self, email: &str) -> Result<bool, DbErr> {
        Ok(User::find().filter(entity::user::Column::Email.eq(email)).count(&self.db).await? > 0)
    }

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<UserModel, DbErr> {
        User::find_by_id(*id).one(&self.db).await?.ok_or_else(|| DbErr::RecordNotFound("User does not exist".into()))
    }

    pub async fn get_user_by_email(&self, email: String) -> Result<UserModel, DbErr> {
        User::find().filter(entity::user::Column::Email.eq(email)).one(&self.db).await?
            .ok_or_else(|| DbErr::RecordNotFound("User does not exist".into()))
    }

    pub async fn get_user_token(&self, id: Uuid) -> Result<String, DbErr> {
        Ok(self.get_user_by_id(&id).await?.token)
    }

    pub async fn regenerate_user_token(&self, user_id: &Uuid) -> Result<String, DbErr> {
        let user = self.get_user_by_id(user_id).await?;
        let token = new_token(TokenType::User);
        let encrypted = encrypt(&token).map_err(|_| DbErr::RecordNotUpdated)?;
        let mut am: UserActive = user.into();
        am.token = Set(encrypted);
        am.updated_at = Set(Utc::now());
        am.update(&self.db).await?;
        Ok(token)
    }

    /// Signup: creates personal team + user in one txn.
    pub async fn create_user(&self, payload: user::DBUserCreate) -> Result<Uuid, AppError> {
        if self.user_exists_by_email(&payload.email).await? { return Err(AppError::AlreadyExists); }
        let uid = token::new_id();
        let team_id = token::new_id();
        let now = Utc::now();
        let txn = self.db.begin().await?;

        Team::insert(TeamActive {
            id: Set(team_id),
            name: Set(format!("{}'s Team", payload.name)),
            owner: Set(uid),
            created_at: Set(now),
            updated_at: Set(now),
        }).exec(&txn).await?;

        User::insert(UserActive {
            id: Set(uid),
            name: Set(payload.name),
            email: Set(payload.email),
            token: Set(payload.token),
            team_id: Set(team_id),
            created_at: Set(now),
            updated_at: Set(now),
        }).exec(&txn).await?;

        txn.commit().await?;
        Ok(uid)
    }

    pub async fn update_user_name(&self, user_id: Uuid, name: String) -> Result<(), DbErr> {
        let mut am: UserActive = self.get_user_by_id(&user_id).await?.into();
        am.name = Set(name);
        am.updated_at = Set(Utc::now());
        am.update(&self.db).await.map(|_| ())
    }

    pub async fn update_user_email(&self, user_id: Uuid, email: String) -> Result<(), DbErr> {
        if self.user_exists_by_email(&email).await? { return Err(DbErr::RecordNotUpdated); }
        let mut am: UserActive = self.get_user_by_id(&user_id).await?.into();
        am.email = Set(email);
        am.updated_at = Set(Utc::now());
        am.update(&self.db).await.map(|_| ())
    }

    pub async fn set_user_team(&self, user_id: Uuid, team_id: Uuid) -> Result<(), DbErr> {
        Team::find_by_id(team_id).one(&self.db).await?.ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))?;
        let mut am: UserActive = self.get_user_by_id(&user_id).await?.into();
        am.team_id = Set(team_id);
        am.updated_at = Set(Utc::now());
        am.update(&self.db).await.map(|_| ())
    }

    pub async fn get_team_for_user(&self, user_id: Uuid) -> Result<TeamModel, DbErr> {
        let u = self.get_user_by_id(&user_id).await?;
        Team::find_by_id(u.team_id).one(&self.db).await?.ok_or_else(|| DbErr::RecordNotFound("Team not found".into()))
    }



    pub async fn list_users_in_team_paginated(&self, team_id: Uuid, page: u64, per_page: u64)
        -> Result<(Vec<UserModel>, u64), DbErr> {
        let finder = User::find().filter(entity::user::Column::TeamId.eq(team_id));
        let total = finder.clone().count(&self.db).await?;
        let items = finder.paginate(&self.db, per_page).fetch_page(page).await?;
        Ok((items, total))
    }

    /// Prevent deleting if the user is listed as team.owner.
    pub async fn delete_user_safe(&self, user_id: Uuid) -> Result<(), DbErr> {
        let owning = Team::find().filter(entity::team::Column::Owner.eq(user_id)).count(&self.db).await?;
        if owning > 0 { return Err(DbErr::RecordNotUpdated); }
        if let Some(u) = User::find_by_id(user_id).one(&self.db).await? {
            let am: UserActive = u.into();
            am.delete(&self.db).await.map(|_| ())
        } else { Ok(()) }
    }
}
