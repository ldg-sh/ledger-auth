use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "team")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub owner: Uuid,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

// Many-to-many via user_team
impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::user_team::Relation::Team.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
