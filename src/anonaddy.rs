use async_trait::async_trait;
use serde::Deserialize;

use crate::email_alias::{Alias, AliasService};

#[derive(Deserialize, Debug)]
pub struct Account {
    id: String,
    username: String,
    from_name: String,
    email_subject: String,
    banner_location: String,
    bandwidth: i16,
    username_count: i8,
    username_limit: i8,
    default_recipient_id: String,
    default_alias_domain: String,
    default_alias_format: String,
    subscription: String,
    subscription_ends_at: Option<i8>,
    bandwidth_limit: i8,
    recipient_count: i8,
    recipient_limit: i8,
    active_domain_count: i8,
    active_domain_limit: i8,
    active_shared_domain_alias_count: i8,
    active_shared_domain_alias_limit: i8,
    total_emails_forwarded: String,
    total_emails_blocked: String,
    total_emails_replied: String,
    total_emails_sent: String,
    created_at: String,
    updated_at: String,
}

#[derive(Deserialize, Debug)]
pub struct AnonAddyAlias {
    pub id: String,
    pub user_id: String,
    pub aliasable_id: Option<String>,
    pub aliasable_type: Option<String>,
    pub local_part: String,
    pub extension: Option<String>,
    pub domain: String,
    pub email: String,
    pub active: bool,
    pub description: Option<String>,
    pub emails_forwarded: i8,
    pub emails_blocked: i8,
    pub emails_replied: i8,
    pub emails_sent: i8,
    pub recipients: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Alias for AnonAddyAlias {
    fn is_active(&self) -> bool {
        self.active
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

#[derive(Deserialize, Debug)]
pub struct AnonAddyResponse<T> {
    pub data: Vec<T>,
}

pub struct AnonAddy<'a> {
    client: &'a reqwest::Client,
    token: String,
    host: String,
}

impl<'a> AnonAddy<'a> {
    pub fn new(client: &'a reqwest::Client, token: String) -> Self {
        AnonAddy {
            client,
            token,
            host: "https://app.anonaddy.com".to_string(),
        }
    }
}

#[async_trait]
impl<'a> AliasService for AnonAddy<'a> {
    async fn get_aliases(&self) -> Result<Vec<Box<dyn Alias>>, Box<dyn std::error::Error>> {
        let aliases = self
            .client
            .get(format!("{}/api/v1/aliases", &(self.host)))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", &(self.token)))
            .send()
            .await?
            .json::<AnonAddyResponse<AnonAddyAlias>>()
            .await?;
        let boxed = aliases
            .data
            .into_iter()
            .map(|alias| {
                let boxed_alias: Box<dyn Alias> = Box::new(alias);
                boxed_alias
            })
            .collect();
        Ok(boxed)
    }

    async fn deactivate_alias(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        print!("Deactivating {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    async fn get_aliases_returns_multiple_aliases() {
        let server = MockServer::start();
        let aliases_mock = server.mock(|when, then| {
            let response = std::fs::read_to_string("resources/test/anonaddy_aliases.json");
            when.method(GET).path("/api/v1/aliases");
            then.status(200)
                .header("content-type", "application/json")
                .body(response.unwrap());
        });

        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let aliases = anonaddy.get_aliases().await;

        assert_eq!(aliases.unwrap().len(), 2);

        aliases_mock.assert();
    }

    #[tokio::test]
    async fn deactivate_alias_returns_ok() {
        let server = MockServer::start();
        let aliases_mock = server.mock(|when, then| {
            let response = std::fs::read_to_string("resources/test/anonaddy_aliases.json");
            when.method(DELETE).path("/api/v1/aliases/");
            then.status(200)
                .header("content-type", "application/json")
                .body(response.unwrap());
        });

        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let aliases = anonaddy.deactivate_alias("test-id".to_string()).await;

        assert_eq!(aliases.unwrap(), ());

        aliases_mock.assert();
    }
}
