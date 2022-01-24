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
    pub fn new(client: &'a reqwest::Client, token: String) -> Self {
        HIBP {
            client,
            token,
            host: "https://haveibeenpwned.com".to_string(),
        }
    }

    pub async fn get_breaches(&self) -> Result<Vec<Breach>, reqwest::Error> {
        let breaches = self
            .client
            .get(format!("{}/api/v3/breachedaccount/", &(self.host)))
            .header("hibp-api-key", &(self.token))
            .send()
            .await?
            .json::<Vec<Breach>>()
            .await?;
        return Ok(breaches);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

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
