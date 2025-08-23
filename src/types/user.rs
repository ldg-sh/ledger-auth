use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct DBUserCreate {
    pub name: String,
    pub email: String,
    pub token: String,
}
