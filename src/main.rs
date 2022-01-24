use dotenv::dotenv;
use email_alias::AliasService;

mod anonaddy;
mod email_alias;
mod hibp;

fn get_alias_service(client: &reqwest::Client) -> Box<dyn AliasService + '_> {
    let anonaddy_token_env = std::env::var("ANONADDY_TOKEN");
    let anonaddy_token = anonaddy_token_env.expect("Please provide ANONADDY_TOKEN");
    let anonaddy = anonaddy::AnonAddy::new(client, anonaddy_token);
    Box::new(anonaddy)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_file = dotenv();
    if env_file.is_err() {
        println!("Unable to find .env file");
    }
    let client = reqwest::Client::new();

    let alias_service = get_alias_service(&client);

    let hibp_token_env = std::env::var("HIBP_TOKEN");
    let hibp_token = hibp_token_env.expect("Please provide HIBP_TOKEN");
    let hibp = hibp::HIBP::new(&client, hibp_token);

    let aliases = alias_service.get_aliases().await?;
    for alias in aliases {
        if alias.is_active() {
            println!("{:#?}", alias);
            let breaches = hibp.get_breaches().await?;
            if breaches.len() > 0 {
                println!("{:#?}", breaches);
                alias_service.deactivate_alias(alias.get_id()).await?;
            }
        }
    }
    Ok(())
}
