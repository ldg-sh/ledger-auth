use crate::db::postgres_service::PostgresService;
use sea_orm::{EntityTrait, ColumnTrait, QueryFilter, Set, ActiveModelTrait};
use crate::types::item::{CreateItem, UpdateItem};
use uuid::Uuid;

impl PostgresService {
    pub async fn get_all_items(&self) -> Option<Vec<entity::item::Model>> {
        use entity::item::{ Entity as ItemData };

        let item_data = ItemData::find()
            .all(&self.db)
            .await
            .ok()?;

        if item_data.is_empty() {
            return None;
        }

        Some(item_data)
    }

    pub async fn get_item_by_id(&self, id: &str) -> Option<entity::item::Model> {
        use entity::item::{ Column, Entity as ItemData };

        let item_data = ItemData::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .ok()?;

        if item_data.is_none() {
            return None;
        }

        item_data
    }

    pub async fn create_item(&self, item: CreateItem) -> Option<entity::item::Model> {
        use entity::item::{Entity as ItemData, Column};
        use chrono::Utc;

        let id  = Uuid::new_v4().to_string();
        let now = Utc::now();

        let new_item = entity::item::ActiveModel {
            id: Set(id.clone()),
            name: Set(item.name),
            created_at: Set(now.to_string()),
            updated_at: Set(now.to_string()),
            ..Default::default()
        };

        ItemData::insert(new_item).exec(&self.db).await.ok()?;

        ItemData::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .ok()?
    }


    pub async fn update_item(
        &self,
        id: &str,
        patch: UpdateItem,
    ) -> Option<entity::item::Model> {
        use entity::item::{Entity as ItemData, Column};

        // fetch the current row
        let current = ItemData::find()
            .filter(Column::Id.eq(id))
            .one(&self.db)
            .await
            .ok()??;

        // convert to ActiveModel
        let mut model: entity::item::ActiveModel = current.into();

        patch.name.map(|v| model.name = Set(v));

        model.update(&self.db).await.ok()
    }

    pub async fn delete_item(&self, id: &str) -> Option<()> {
        use entity::item::{ Entity as ItemData };

        ItemData::delete_by_id(id).exec(&self.db).await.ok()?;

        Some(())
    }

    pub async fn delete_all_items(&self) -> Option<()> {
        use entity::item::{ Entity as ItemData };

        ItemData::delete_many().exec(&self.db).await.ok()?;

        Some(())
    }
}
