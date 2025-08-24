use crate::types::token::TokenType;
use crate::{db::postgres_service::PostgresService, utils::token::new_token};
use crate::types::user;
use crate::utils::token::{self, encrypt};
use chrono::Utc;
use entity::user::{ActiveModel, Entity as User, Model as UserModel};
use sea_orm::{ColumnTrait, DbErr, EntityTrait, ActiveModelTrait, QueryFilter, Set};
use uuid::Uuid;
use crate::types::error::AppError;


impl PostgresService {
    // *** CREATE ***
    /// Create a user return their user ID
    pub async fn create_user(&self, user: user::DBUserCreate) -> Result<Uuid, AppError> {
        let uid = token::new_id();

        if (self.get_user_by_email(user.email.clone()).await).is_ok() {
            return Err(AppError::AlreadyExists)
        };

        let user = entity::user::ActiveModel {
            id: Set(uid),
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

    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<UserModel, DbErr> {
        let user = match User::find_by_id(*id).one(&self.db).await? {
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

    /// Regenerates and updates a user's token in the database.
    ///
    /// This function:
    /// 1. Fetches the user record by `Uuid`.
    /// 2. Generates a new raw token using [`new_token`].
    /// 3. Encrypts the token for storage.
    /// 4. Updates the user's `token` field in the database with the encrypted value.
    /// 5. Returns the new **raw** token (to be returned once to the client).
    ///
    /// # Arguments
    /// * `user` - The [`Uuid`] of the user whose token should be regenerated.
    ///
    /// # Returns
    /// * `Ok(String)` - The new raw token if the operation succeeds.
    /// * `Err(DbErr)` - If the user cannot be found, the token encryption fails,
    ///   or the database update does not succeed.
    ///
    /// # Errors
    /// * [`DbErr::RecordNotFound`] if the user does not exist.
    /// * [`DbErr::RecordNotUpdated`] if the encryption fails or the update does not persist.
    /// * Any other [`DbErr`] returned from the underlying database call.
    ///
    /// # Notes
    /// The returned token is **not encrypted**; store or transmit it securely,
    /// as this is the only chance to access the plain token.
    ///
    /// # Example
    /// ```ignore
    /// let user_id = Uuid::new_v4();
    /// match service.regenerate_user_token(user_id).await {
    ///     Ok(token) => println!("new token: {}", token),
    ///     Err(e) => eprintln!("failed: {}", e),
    /// }
    /// ```
    pub async fn regenerate_user_token(&self, user: &Uuid) -> Result<String, DbErr> {
        let user = self.get_user_by_id(&user).await?;

        let token = new_token(TokenType::User);

        let encrypted_token = match encrypt(&token) {
            Ok(encrypted_token) => encrypted_token,
            Err(_) => {
                return Err(DbErr::RecordNotUpdated)
            },
        };

        let mut am: ActiveModel = user.into();
        am.token = Set(encrypted_token);
        am.updated_at = Set(Utc::now());
        am.update(&self.db).await?;

        Ok(token)
    }
}
