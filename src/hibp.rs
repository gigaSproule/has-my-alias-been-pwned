use std::fmt::Display;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Breach {
    Name: String,
    Title: String,
    Domain: String,
    BreachDate: String,
    AddedDate: String,
    ModifiedDate: String,
    PwnCount: i32,
    Description: String,
    DataClasses: Vec<String>,
    IsVerified: bool,
    IsFabricated: bool,
    IsSensitive: bool,
    IsRetired: bool,
    IsSpamList: bool,
    LogoPath: String,
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

    pub async fn get_breaches(&self) -> Result<Vec<Breach>, Box<dyn std::error::Error>> {
        let response = self
            .client
            .get(format!("{}/api/v3/breachedaccount/", &(self.host)))
            .header("hibp-api-key", &(self.token))
            .send()
            .await?;
        if response.status() != 200 {
            return Err(Box::new(HIBPError::new(
                "Failed to get breaches.".to_string(),
            )));
        }
        let breaches = response.json::<Vec<Breach>>().await?;
        return Ok(breaches);
    }
}

#[derive(Debug, Clone)]

pub struct HIBPError {
    pub message: String,
}

impl HIBPError {
    pub fn new(message: String) -> Self {
        HIBPError { message }
    }
}

impl Display for HIBPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HIBPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    #[tokio::test]
    #[should_panic(expected = "Please provide HIBP_TOKEN: NotPresent")]
    async fn new_throw_error_if_token_variable_not_set() {
        let client = reqwest::Client::new();
        std::env::remove_var("HIBP_TOKEN");
        HIBP::new(&client);
    }

    #[tokio::test]
    async fn new_return_instance_if_token_variable_empty() {
        let client = reqwest::Client::new();
        std::env::set_var("HIBP_TOKEN", "");

        let hibp = HIBP::new(&client);

        assert_eq!(hibp.client as *const _, &client as *const _);
        assert_eq!(hibp.token, "");
        assert_eq!(hibp.host, "https://haveibeenpwned.com".to_string());
    }

    #[tokio::test]
    async fn new_return_instance_if_token_variable_has_value() {
        let client = reqwest::Client::new();
        std::env::set_var("HIBP_TOKEN", "test-token");

        let hibp = HIBP::new(&client);

        assert_eq!(hibp.client as *const _, &client as *const _);
        assert_eq!(hibp.token, "test-token");
        assert_eq!(hibp.host, "https://haveibeenpwned.com".to_string());
    }

    #[tokio::test]
    async fn get_breaches_returns_error_for_no_response() {
        let client = reqwest::Client::new();
        let hibp = HIBP {
            client: &client,
            token: "test-token".to_string(),
            host: "http://localhost".to_string(),
        };

        let response = hibp.get_breaches().await;

        assert!(response.is_err());
        let error = response.unwrap_err();
        let actual_error: &reqwest::Error = match error.downcast_ref::<reqwest::Error>() {
            Some(error) => error,
            None => panic!("Error returned was not an reqwest::Error!"),
        };
        assert!(actual_error.is_request());
    }

    #[tokio::test]
    async fn get_breaches_returns_error_for_non_ok() {
        let server = MockServer::start();
        let breaches_mock = server.mock(|when, then| {
            when.method(GET).path("/api/v3/breachedaccount/");
            then.status(400).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let hibp = HIBP {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = hibp.get_breaches().await;

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
    async fn get_breaches_returns_error_for_no_body() {
        let server = MockServer::start();
        let breaches_mock = server.mock(|when, then| {
            when.method(GET).path("/api/v3/breachedaccount/");
            then.status(200).header("content-type", "application/json");
        });

        let client = reqwest::Client::new();
        let hibp = HIBP {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let response = hibp.get_breaches().await;

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
    async fn get_breaches_returns_multiple_breaches() {
        let server = MockServer::start();
        let breaches_mock = server.mock(|when, then| {
            let response = std::fs::read_to_string("resources/test/hibp_breaches.json");
            when.method(GET).path("/api/v3/breachedaccount/");
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

        let breaches = hibp.get_breaches().await;

        assert_eq!(breaches.unwrap().len(), 2);

        breaches_mock.assert();
    }
}
