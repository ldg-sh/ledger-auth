use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "user_team")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: Uuid,
    #[sea_orm(primary_key)]
    pub team_id: Uuid,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to   = "super::user::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    User,

    #[sea_orm(
        belongs_to = "super::team::Entity",
        from = "Column::TeamId",
        to   = "super::team::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Team,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef { Relation::User.def() }
}

impl Related<super::team::Entity> for Entity {
    fn to() -> RelationDef { Relation::Team.def() }
}

impl ActiveModelBehavior for ActiveModel {}
