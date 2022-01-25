use dotenv::dotenv;
use email_alias::AliasService;

mod anonaddy;
mod email_alias;
mod hibp;

fn get_alias_service(client: &reqwest::Client) -> Box<dyn AliasService + '_> {
    let anonaddy = anonaddy::AnonAddy::new(client);
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

    let hibp = hibp::HIBP::new(&client);

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
