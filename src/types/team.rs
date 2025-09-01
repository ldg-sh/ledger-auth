use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct RTeamCreate {
    pub owner: Uuid,
    pub name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamCreateRes {
    pub id: String,
    pub message: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RTeamAddUser {
    pub team: Uuid,
    pub user: Uuid
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TeamAddUserRes {
    pub message: String
}
