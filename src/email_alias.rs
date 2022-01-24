use std::fmt::Debug;

use async_trait::async_trait;
pub trait Alias: Debug {
    fn is_active(&self) -> bool;
    fn get_id(&self) -> String;
}

#[async_trait]
pub trait AliasService {
    async fn get_aliases(&self) -> Result<Vec<Box<dyn Alias>>, Box<dyn std::error::Error>>;

    async fn deactivate_alias(&self, id: String) -> Result<(), Box<dyn std::error::Error>>;
}
