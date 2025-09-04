use chrono::Utc;
use uuid::Uuid;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, Set,
    TransactionTrait,
};
use crate::{types::error::AppError, utils::token};
use crate::db::postgres_service::PostgresService;
use entity::team_invite::{ActiveModel as InviteActive, Entity as Invite, Model as InviteModel};

impl PostgresService {
    pub async fn create_invite(
        &self,
        team_id: Uuid,
        user_id: Uuid,
        invited_by: Uuid,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<String, DbErr> {
        if self.has_active_invite(team_id, user_id).await? {
            return Err(DbErr::RecordNotInserted);
        }
        let id = token::new_nanoid(10);
        let now = Utc::now();
        Invite::insert(InviteActive {
            id: Set(id.clone()),
            team_id: Set(team_id),
            user_id: Set(user_id),
            invited_by: Set(invited_by),
            status: Set(false),
            expires_at: Set(expires_at),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec(&self.db)
        .await?;
        Ok(id)
    }

    pub async fn get_invite(&self, id: &str) -> Result<InviteModel, DbErr> {
        Invite::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or(DbErr::RecordNotFound("Invite not found".into()))
    }

    pub async fn list_pending_invites_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<InviteModel>, DbErr> {
        Invite::find()
            .filter(entity::team_invite::Column::UserId.eq(user_id))
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.gt(Utc::now()))
            .all(&self.db)
            .await
    }

    pub async fn list_pending_invites_for_team(
        &self,
        team_id: Uuid,
    ) -> Result<Vec<InviteModel>, DbErr> {
        Invite::find()
            .filter(entity::team_invite::Column::TeamId.eq(team_id))
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.gt(Utc::now()))
            .all(&self.db)
            .await
    }

    pub async fn has_active_invite(&self, team_id: Uuid, user_id: Uuid) -> Result<bool, DbErr> {
        Ok(Invite::find()
            .filter(entity::team_invite::Column::TeamId.eq(team_id))
            .filter(entity::team_invite::Column::UserId.eq(user_id))
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.gt(Utc::now()))
            .count(&self.db)
            .await? > 0)
    }

    pub async fn accept_invite(&self, invite_id: &str) -> Result<(), DbErr> {
        let txn = self.db.begin().await?;

        let inv = Invite::find_by_id(invite_id.to_string())
            .one(&txn)
            .await?
            .ok_or(DbErr::RecordNotFound("Invite not found".into()))?;

        if inv.expires_at <= Utc::now() || inv.status {
            txn.rollback().await?;
            return Err(DbErr::RecordNotUpdated);
        }

        let mut am: InviteActive = inv.into();
        am.status = Set(true);
        am.updated_at = Set(Utc::now());
        am.update(&txn).await?;

        txn.commit().await?;
        Ok(())
    }

    /// Hard-delete all expired *pending* invites.
    pub async fn expire_invites(&self) -> Result<u64, DbErr> {
        let res = Invite::delete_many()
            .filter(entity::team_invite::Column::Status.eq(false))
            .filter(entity::team_invite::Column::ExpiresAt.lte(Utc::now()))
            .exec(&self.db)
            .await?;
        Ok(res.rows_affected)
    }

    /// Hard-delete a specific invite (cancel).
    pub async fn delete_invite(&self, invite_id: &str) -> Result<(), DbErr> {
        let res = Invite::delete_by_id(invite_id.to_string()).exec(&self.db).await?;
        if res.rows_affected == 0 {
            return Err(DbErr::RecordNotFound("Invite not found".into()));
        }
        Ok(())
    }
}
