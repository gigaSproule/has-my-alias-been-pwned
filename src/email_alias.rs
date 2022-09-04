use std::fmt::{Debug, Display};

use async_trait::async_trait;
pub trait Alias: Debug {
    fn is_active(&self) -> bool;
    fn get_id(&self) -> &str;
    fn get_email(&self) -> &str;
    fn get_description(&self) -> Option<&str>;
}

#[async_trait]
pub trait AliasService {
    async fn get_aliases(&self) -> Result<Vec<Box<dyn Alias>>, Box<dyn std::error::Error>>;

    async fn deactivate_alias(&self, id: &str) -> Result<(), Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
pub struct AliasError {
    pub message: String,
}

impl AliasError {
    pub fn new(message: String) -> Self {
        AliasError { message }
    }
}

impl Display for AliasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AliasError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
