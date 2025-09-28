use crate::db::postgres_service::PostgresService;
use crate::{types::error::AppError, utils::token};
use chrono::Utc;
use entity::team_invite::{ActiveModel as InviteActive, Entity as Invite, Model as InviteModel};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set, SqlErr,
    TransactionTrait,
};
use uuid::Uuid;

impl PostgresService {
    pub async fn create_invite(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        invited_by: Uuid,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<String, AppError> {
        // Validate related records so we can send domain errors instead of 500s
        self.get_team(team_id).await?;
        self.get_user_by_id(&user_id).await?;
        self.get_user_by_id(&invited_by).await?;

        if self.has_active_invite(team_id, user_id).await? {
            return Err(AppError::AlreadyExists);
        }
        let id = token::new_nanoid(10);
        let now = Utc::now();
        match Invite::insert(InviteActive {
            id: Set(id.clone()),
            team_id: Set(team_id),
            user_id: Set(user_id),
            invited_by: Set(invited_by),
            status: Set(false),
            expires_at: Set(expires_at),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(&self.database_connection)
        .await
        {
            Ok(_) => Ok(id),
            Err(err) => {
                if let Some(sql_err) = err.sql_err() {
                    return match sql_err {
                        SqlErr::UniqueConstraintViolation(_) => Err(AppError::AlreadyExists),
                        SqlErr::ForeignKeyConstraintViolation(_) => {
                            Err(AppError::BadRequest("Related record missing".to_string()))
                        }
                        _ => Err(err.into()),
                    };
                }
                Err(err.into())
            }
        }
    }

    pub async fn get_invite(&self, id: &str) -> Result<InviteModel, AppError> {
        Ok(Invite::find_by_id(id.to_string())
            .one(&self.database_connection)
            .await?
            .ok_or(DbErr::RecordNotFound("Invite not found".into()))?)
    }

    pub async fn list_pending_invites_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<InviteModel>, AppError> {
        Ok(Invite::find()
            .filter(entity::team_invite::Column::UserId.eq(user_id))
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.gt(Utc::now()))
            .all(&self.database_connection)
            .await?)
    }

    pub async fn list_pending_invites_for_team(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<InviteModel>, AppError> {
        Ok(Invite::find()
            .filter(entity::team_invite::Column::TeamId.eq(team_id))
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.gt(Utc::now()))
            .all(&self.database_connection)
            .await?)
    }

    pub async fn has_active_invite(&self, team_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        Ok(Invite::find()
            .filter(entity::team_invite::Column::TeamId.eq(team_id))
            .filter(entity::team_invite::Column::UserId.eq(user_id))
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.gt(Utc::now()))
            .count(&self.database_connection)
            .await?
            > 0)
    }

    pub async fn accept_invite(&self, invite_id: &str) -> Result<(), AppError> {
        let txn = self.database_connection.begin().await?;

        let inv = Invite::find_by_id(invite_id.to_string())
            .one(&txn)
            .await?
            .ok_or(DbErr::RecordNotFound("Invite not found".into()))?;

        if inv.expires_at <= Utc::now() || inv.status {
            txn.rollback().await?;
            return Err(AppError::Db(DbErr::RecordNotUpdated));
        }

        let mut am: InviteActive = inv.into();
        am.status = Set(true);
        am.updated_at = Set(Utc::now());
        am.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    /// Hard-delete all expired *pending* invites.
    pub async fn expire_invites(&self) -> Result<u64, AppError> {
        let res = Invite::delete_many()
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.lte(Utc::now()))
            .exec(&self.database_connection)
            .await?;
        Ok(res.rows_affected)
    }

    /// Hard-delete a specific invite (cancel).
    pub async fn delete_invite(&self, invite_id: &str) -> Result<(), AppError> {
        let res = Invite::delete_by_id(invite_id.to_string())
            .exec(&self.database_connection)
            .await?;
        if res.rows_affected == 0 {
            return Err(AppError::Db(DbErr::RecordNotFound(
                "Invite not found".into(),
            )));
        }
        Ok(())
    }
}
