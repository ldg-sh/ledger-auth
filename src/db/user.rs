use crate::db::postgres_service::PostgresService;
use entity::user::{Entity as User, Model as UserModel};
use sea_orm::{DbErr, EntityTrait, Set, QueryFilter, QuerySelect, ColumnTrait, ModelTrait};
use crate::types::user;
use chrono::Utc;
use crate::utils::token;
use uuid::Uuid;
use crate::types::token::TokenType;

impl PostgresService {
    // *** CREATE ***
    /// Create a user return their user ID
    pub async fn create_user(&self, user: user::DBUserCreate) -> Result<Uuid, DbErr> {
        let uid = token::new_id();

        match self.get_user_by_email(user.email.clone()).await {
            Ok(exists) => {
                return Ok(exists.id)
            },
            Err(_) => {},
        };

        let user = entity::user::ActiveModel {
            id: Set(uid.clone()),
            name: Set(user.name),
            email: Set(user.email),
            token: Set(user.token),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now())
        };

        User::insert(user)
            .exec(&self.db)
            .await?;

        Ok(uid)
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<UserModel, DbErr> {
        let user = match User::find_by_id(id).one(&self.db).await? {
            Some(user) => user,
            None => {
                return Err(DbErr::RecordNotFound("User does not exist".to_string()));
            }
        };

        Ok(user)
    }

    pub async fn get_user_by_email(&self, mail: String) -> Result<UserModel, DbErr> {
        let user = match User::find().filter(entity::user::Column::Email.eq(mail)).one(&self.db).await? {
            Some(user) => user,
            None => {
                return Err(DbErr::RecordNotFound("User does not exist".to_string()))
            },
        };

        Ok(user)
    }

    pub async fn get_user_token(&self, id: Uuid) -> Result<String, DbErr> {
        let maybe_user = User::find()
            .filter(entity::user::Column::Id.eq(id))
            .one(&self.db)
            .await?;

        let user = match maybe_user {
            Some(user) => user,
            None => return Err(DbErr::RecordNotFound("User does not exist".to_string())),
        };

        Ok(user.token)
    }
}
