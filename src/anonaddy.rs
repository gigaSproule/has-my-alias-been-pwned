use async_trait::async_trait;
use log::info;
use serde::Deserialize;

use crate::email_alias::{Alias, AliasError, AliasService};

#[derive(Deserialize, Debug)]
pub struct Account {
    id: String,
    username: String,
    from_name: String,
    email_subject: String,
    banner_location: String,
    bandwidth: i16,
    username_count: i16,
    username_limit: i16,
    default_recipient_id: String,
    default_alias_domain: String,
    default_alias_format: String,
    subscription: String,
    subscription_ends_at: Option<i16>,
    bandwidth_limit: i16,
    recipient_count: i16,
    recipient_limit: i16,
    active_domain_count: i16,
    active_domain_limit: i16,
    active_shared_domain_alias_count: i16,
    active_shared_domain_alias_limit: i16,
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
    pub emails_forwarded: i16,
    pub emails_blocked: i16,
    pub emails_replied: i16,
    pub emails_sent: i16,
    pub recipients: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Alias for AnonAddyAlias {
    fn is_active(&self) -> bool {
        self.active
    }

    fn get_id(&self) -> &str {
        self.id.as_ref()
    }

    fn get_email(&self) -> &str {
        self.email.as_ref()
    }

    fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
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
    /// Creates a new instance to query against an AnonAddy instance.
    ///
    /// For this to work, a `ANONADDY_TOKEN` environment variable must be set. If it is not set, this will panic.
    /// By default, this will use `app.anonaddy.com`, but this can be overriden by setting the `ANONADDY_HOST` environment variable to the desired instance URL.
    ///
    /// # Examples
    /// Only providing the token:
    /// ```
    /// let client = reqwest::Client::new();
    /// std::env::set_var("ANONADDY_TOKEN", "test-token");
    /// let anonaddy = ANONADDY_TOKEN::new(&client);
    /// ```
    /// Providing the token and the host:
    /// ```
    /// let client = reqwest::Client::new();
    /// std::env::set_var("ANONADDY_TOKEN", "test-token");
    /// std::env::set_var("ANONADDY_HOST", "https://my-anonaddy-instance.com");
    /// let anonaddy = ANONADDY_TOKEN::new(&client);
    /// ```
    pub fn new(client: &'a reqwest::Client) -> Self {
        let token = std::env::var("ANONADDY_TOKEN").expect("Please provide ANONADDY_TOKEN");
        let host = std::env::var("ANONADDY_HOST")
            .unwrap_or_else(|_| "https://app.anonaddy.com".to_string());
        AnonAddy {
            client,
            token,
            host,
        }
    }
}

#[async_trait]
impl<'a> AliasService for AnonAddy<'a> {
    async fn get_aliases(&self) -> Result<Vec<Box<dyn Alias>>, Box<dyn std::error::Error>> {
        info!("Getting aliases from AnonAddy.");
        let response = self
            .client
            .get(format!("{}/api/v1/aliases", &(self.host)))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", &(self.token)))
            .send()
            .await?;
        if response.status() != 200 {
            return Err(Box::new(AliasError::new(
                "Failed to get aliases.".to_string(),
            )));
        }
        let aliases = response.json::<AnonAddyResponse<AnonAddyAlias>>().await?;
        let boxed: Vec<Box<dyn Alias>> = aliases
            .data
            .into_iter()
            .map(|alias| {
                let boxed_alias: Box<dyn Alias> = Box::new(alias);
                boxed_alias
            })
            .collect();
        info!("Retrieved {} aliases.", boxed.len());
        Ok(boxed)
    }

    async fn deactivate_alias(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Deactivating alias {}.", id);
        let response = self
            .client
            .delete(format!("{}/api/v1/active-aliases/{}", &(self.host), id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", &(self.token)))
            .send()
            .await?;
        if response.status() != 204 {
            return Err(Box::new(AliasError::new(format!(
                "Failed to deactivate alias {}.",
                id
            ))));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;

    use super::*;

    #[tokio::test]
    #[should_panic(expected = "Please provide ANONADDY_TOKEN: NotPresent")]
    async fn new_throw_error_if_token_variable_not_set() {
        let client = reqwest::Client::new();
        std::env::remove_var("ANONADDY_TOKEN");
        std::env::remove_var("ANONADDY_HOST");
        AnonAddy::new(&client);
    }

    #[tokio::test]
    async fn new_return_instance_if_token_variable_empty() {
        let client = reqwest::Client::new();
        std::env::set_var("ANONADDY_TOKEN", "");
        std::env::remove_var("ANONADDY_HOST");

        let anonaddy = AnonAddy::new(&client);

        assert_eq!(anonaddy.client as *const _, &client as *const _);
        assert_eq!(anonaddy.token, "");
        assert_eq!(anonaddy.host, "https://app.anonaddy.com".to_string());
    }

    #[tokio::test]
    async fn new_return_instance_if_token_variable_has_value() {
        let client = reqwest::Client::new();
        std::env::set_var("ANONADDY_TOKEN", "test-token");
        std::env::remove_var("ANONADDY_HOST");

        let anonaddy = AnonAddy::new(&client);

        assert_eq!(anonaddy.client as *const _, &client as *const _);
        assert_eq!(anonaddy.token, "test-token");
        assert_eq!(anonaddy.host, "https://app.anonaddy.com".to_string());
    }

    #[tokio::test]
    async fn new_return_instance_with_custom_host_if_provided() {
        let client = reqwest::Client::new();
        std::env::set_var("ANONADDY_TOKEN", "test-token");
        std::env::set_var("ANONADDY_HOST", "https://my-anonaddy-instance.com");

        let anonaddy = AnonAddy::new(&client);

        assert_eq!(anonaddy.client as *const _, &client as *const _);
        assert_eq!(anonaddy.token, "test-token");
        assert_eq!(
            anonaddy.host,
            "https://my-anonaddy-instance.com".to_string()
        );
    }

    #[tokio::test]
    async fn get_aliases_returns_error_for_no_response() {
        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: "https://localhost".to_string(),
        };

        let response = anonaddy.get_aliases().await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &reqwest::Error = match error.downcast_ref::<reqwest::Error>() {
            Some(error) => error,
            None => panic!("Error returned was not an reqwest::Error!"),
        };
        assert!(actual_error.is_request());
    }

    #[tokio::test]
    async fn get_aliases_returns_error_for_non_ok() {
        let server = MockServer::start();
        let aliases_mock = server.mock(|when, then| {
            when.method(GET).path("/api/v1/aliases");
            then.status(400).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = anonaddy.get_aliases().await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &AliasError = match error.downcast_ref::<AliasError>() {
            Some(error) => error,
            None => panic!("Error returned was not an AliasError!"),
        };
        assert_eq!(actual_error.message, "Failed to get aliases.");

        aliases_mock.assert();
    }

    #[tokio::test]
    async fn get_aliases_returns_error_for_no_body() {
        let server = MockServer::start();
        let aliases_mock = server.mock(|when, then| {
            when.method(GET).path("/api/v1/aliases");
            then.status(200).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = anonaddy.get_aliases().await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &reqwest::Error = match error.downcast_ref::<reqwest::Error>() {
            Some(error) => error,
            None => panic!("Error returned was not an reqwest::Error!"),
        };
        assert!(actual_error.is_decode());

        aliases_mock.assert();
    }

    #[tokio::test]
    async fn get_aliases_returns_inactive_alias() {
        let server = MockServer::start();
        let aliases_mock = server.mock(|when, then| {
            let response = std::fs::read_to_string("resources/test/anonaddy_inactive_alias.json");
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

        let response = anonaddy.get_aliases().await;

        let aliases = response.unwrap();
        assert_eq!(aliases.len(), 1);
        let alias = aliases.get(0).unwrap();
        assert_eq!(
            alias.get_id(),
            "50c9e585-e7f5-41c4-9016-9014c15454bc-inactive"
        );
        assert_eq!(alias.is_active(), false);

        aliases_mock.assert();
    }

    #[tokio::test]
    async fn get_aliases_returns_active_alis() {
        let server = MockServer::start();
        let aliases_mock = server.mock(|when, then| {
            let response = std::fs::read_to_string("resources/test/anonaddy_active_alias.json");
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

        let response = anonaddy.get_aliases().await;

        let aliases = response.unwrap();
        assert_eq!(aliases.len(), 1);
        let alias = aliases.get(0).unwrap();
        assert_eq!(
            alias.get_id(),
            "50c9e585-e7f5-41c4-9016-9014c15454bc-active"
        );
        assert_eq!(alias.is_active(), true);

        aliases_mock.assert();
    }

    #[tokio::test]
    async fn get_aliases_returns_multiple_aliases() {
        let server = MockServer::start();
        let aliases_mock = server.mock(|when, then| {
            let response = std::fs::read_to_string("resources/test/anonaddy_multiple_aliases.json");
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
    async fn deactivate_alias_returns_error_for_no_response() {
        let alias_id = "test-id";

        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: "http://localhost".to_string(),
        };

        let response = anonaddy.deactivate_alias(alias_id).await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &reqwest::Error = match error.downcast_ref::<reqwest::Error>() {
            Some(error) => error,
            None => panic!("Error returned was not a reqwest::Error!"),
        };
        assert!(actual_error.is_request());
    }

    #[tokio::test]
    async fn deactivate_alias_returns_error_if_status_200() {
        let server = MockServer::start();

        let alias_id = "test-id";
        let aliases_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/api/v1/active-aliases/{}", &alias_id));
            then.status(200).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = anonaddy.deactivate_alias(alias_id).await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &AliasError = match error.downcast_ref::<AliasError>() {
            Some(error) => error,
            None => panic!("Error returned was not an AliasError!"),
        };
        assert_eq!(
            actual_error.message,
            format!("Failed to deactivate alias {}.", alias_id)
        );

        aliases_mock.assert();
    }

    #[tokio::test]
    async fn deactivate_alias_returns_ok() {
        let server = MockServer::start();

        let alias_id = "test-id";
        let aliases_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path(format!("/api/v1/active-aliases/{}", &alias_id));
            then.status(204).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let anonaddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = anonaddy.deactivate_alias(alias_id).await;

        assert!(response.is_ok());

        aliases_mock.assert();
    }
}
