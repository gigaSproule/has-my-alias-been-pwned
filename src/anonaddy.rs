use serde::Deserialize;

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
pub struct Alias {
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

    pub async fn get_aliases(&self) -> Result<AnonAddyResponse<Alias>, reqwest::Error> {
        let aliases = self
            .client
            .get(format!("{}/api/v1/aliases", &(self.host)))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", &(self.token)))
            .send()
            .await?
            .json::<AnonAddyResponse<Alias>>()
            .await?;
        Ok(aliases)
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
        let anonAddy = AnonAddy {
            client: &client,
            token: "test-token".to_string(),
            host: server.url(""),
        };

        let aliases = anonAddy.get_aliases().await;

        assert_eq!(aliases.unwrap().data.len(), 2);

        aliases_mock.assert();
    }
}
