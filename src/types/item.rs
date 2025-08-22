use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateItem {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateItem {
    pub name: Option<String>,
}
