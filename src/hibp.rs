use std::{fmt::Display, thread, time};

use log::debug;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Breach {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Domain")]
    domain: String,
    #[serde(rename = "BreachDate")]
    breach_date: String,
    #[serde(rename = "AddedDate")]
    added_date: String,
    #[serde(rename = "ModifiedDate")]
    modified_date: String,
    #[serde(rename = "PwnCount")]
    pwn_count: i32,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "DataClasses")]
    data_classes: Vec<String>,
    #[serde(rename = "IsVerified")]
    is_verified: bool,
    #[serde(rename = "IsFabricated")]
    is_fabricated: bool,
    #[serde(rename = "IsSensitive")]
    is_sensitive: bool,
    #[serde(rename = "IsRetired")]
    is_retired: bool,
    #[serde(rename = "IsSpamList")]
    is_spam_list: bool,
    #[serde(rename = "LogoPath")]
    logo_path: String,
}

pub struct HIBP<'a> {
    client: &'a reqwest::Client,
    token: String,
    host: String,
}

impl<'a> HIBP<'a> {
    /// Creates a new instance to query against haveibeenpwned.com.
    ///
    /// For this to work, a `HIBP_TOKEN` environment variable must be set. If it is not set, this will panic.
    ///
    /// # Examples
    /// ```
    /// let client = reqwest::Client::new();
    /// std::env::set_var("HIBP_TOKEN", "test-token");
    /// let hibp = HIBP::new(&client);
    /// ```
    pub fn new(client: &'a reqwest::Client) -> Self {
        let token = std::env::var("HIBP_TOKEN").expect("Please provide HIBP_TOKEN");
        HIBP {
            client,
            token,
            host: "https://haveibeenpwned.com".to_string(),
        }
    }

    pub async fn get_breaches(
        &self,
        email_address: &str,
    ) -> Result<Vec<Breach>, Box<dyn std::error::Error>> {
        let url = &format!(
            "{}/api/v3/breachedaccount/{}?truncateResponse=false",
            &(self.host),
            email_address
        );
        let response = self
            .client
            .get(url)
            .header("hibp-api-key", &(self.token))
            .header("user-agent", "has-my-alias-been-pwned")
            .send()
            .await?;
        if response.status() == 404 {
            return Ok(vec![]);
        }
        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .unwrap()
                .to_str()?
                .parse::<u64>()?;
            let duration = time::Duration::from_secs(retry_after);
            debug!("Need to wait {} seconds.", duration.as_secs());
            thread::sleep(duration);
            let response = self
                .client
                .get(url)
                .header("hibp-api-key", &(self.token))
                .header("user-agent", "has-my-alias-been-pwned")
                .send()
                .await?;
            if response.status() == 404 {
                return Ok(vec![]);
            }
            let breaches = response.json::<Vec<Breach>>().await?;
            return Ok(breaches);
        }
        if response.status() != 200 {
            return Err(Box::new(HIBPError::new(
                "Failed to get breaches.".to_string(),
                response.status().as_u16(),
            )));
        }
        let breaches = response.json::<Vec<Breach>>().await?;
        Ok(breaches)
    }
}

#[derive(Debug, Clone)]
pub struct HIBPError {
    pub message: String,
    pub status_code: u16,
}

impl HIBPError {
    pub fn new(message: String, status_code: u16) -> Self {
        HIBPError {
            message,
            status_code,
        }
    }
}

impl Display for HIBPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.status_code, self.message)
    }
}

impl std::error::Error for HIBPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use serial_test::serial;

    use super::*;

    #[tokio::test]
    #[serial]
    #[should_panic(expected = "Please provide HIBP_TOKEN: NotPresent")]
    async fn new_throw_error_if_token_variable_not_set() {
        let client = reqwest::Client::new();
        std::env::remove_var("HIBP_TOKEN");
        HIBP::new(&client);
    }

    #[tokio::test]
    #[serial]
    async fn new_return_instance_if_token_variable_empty() {
        let client = reqwest::Client::new();
        std::env::set_var("HIBP_TOKEN", "");

        let hibp = HIBP::new(&client);

        assert_eq!(hibp.client as *const _, &client as *const _);
        assert_eq!(hibp.token, "");
        assert_eq!(hibp.host, "https://haveibeenpwned.com".to_string());
    }

    #[tokio::test]
    #[serial]
    async fn new_return_instance_if_token_variable_has_value() {
        let client = reqwest::Client::new();
        std::env::set_var("HIBP_TOKEN", "test-token");

        let hibp = HIBP::new(&client);

        assert_eq!(hibp.client as *const _, &client as *const _);
        assert_eq!(hibp.token, "test-token");
        assert_eq!(hibp.host, "https://haveibeenpwned.com".to_string());
    }

    #[tokio::test]
    #[serial]
    async fn get_breaches_returns_error_for_no_response() {
        let client = reqwest::Client::new();
        let hibp = HIBP {
            client: &client,
            token: "test-token".to_string(),
            host: "http://localhost".to_string(),
        };

        let response = hibp.get_breaches("email@email.com").await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &reqwest::Error = match error.downcast_ref::<reqwest::Error>() {
            Some(error) => error,
            None => panic!("Error returned was not an reqwest::Error!"),
        };
        assert!(actual_error.is_request());
    }

    #[tokio::test]
    #[serial]
    async fn get_breaches_returns_error_for_non_ok() {
        let server = MockServer::start();
        let breaches_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/v3/breachedaccount/email@email.com")
                .query_param("truncateResponse", "false")
                .header("hibp-api-key", "test-token")
                .header("user-agent", "has-my-alias-been-pwned");
            then.status(400).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let hibp = HIBP {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = hibp.get_breaches("email@email.com").await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &HIBPError = match error.downcast_ref::<HIBPError>() {
            Some(error) => error,
            None => panic!("Error returned was not an HIBPError!"),
        };
        assert_eq!(actual_error.message, "Failed to get breaches.");

        breaches_mock.assert();
    }

    #[tokio::test]
    #[serial]
    async fn get_breaches_returns_error_for_no_body() {
        let server = MockServer::start();
        let breaches_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/api/v3/breachedaccount/email@email.com")
                .query_param("truncateResponse", "false")
                .header("hibp-api-key", "test-token")
                .header("user-agent", "has-my-alias-been-pwned");
            then.status(200).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let hibp = HIBP {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = hibp.get_breaches("email@email.com").await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &reqwest::Error = match error.downcast_ref::<reqwest::Error>() {
            Some(error) => error,
            None => panic!("Error returned was not an reqwest::Error!"),
        };
        assert!(actual_error.is_decode());

        breaches_mock.assert();
    }

    #[tokio::test]
    #[serial]
    async fn get_breaches_returns_multiple_breaches() {
        let server = MockServer::start();
        let breaches_mock = server.mock(|when, then| {
            let response = std::fs::read_to_string("resources/test/hibp_breaches.json");
            when.method(GET)
                .path("/api/v3/breachedaccount/email@email.com")
                .query_param("truncateResponse", "false")
                .header("hibp-api-key", "test-token")
                .header("user-agent", "has-my-alias-been-pwned");
            then.status(200)
                .header("content-type", "application/json")
                .body(response.unwrap());
        });

        let client = reqwest::Client::new();
        let hibp = HIBP {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let breaches = hibp.get_breaches("email@email.com").await;

        assert_eq!(breaches.unwrap().len(), 2);

        breaches_mock.assert();
    }
}
