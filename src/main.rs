use dotenv::dotenv;

mod anonaddy;
mod hibp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_file = dotenv();
    if env_file.is_err() {
        println!("Unable to find .env file");
    }
    let client = reqwest::Client::new();

    let hibp_token_env = std::env::var("HIBP_TOKEN");
    let hibp_token = hibp_token_env.expect("Please provide HIBP_TOKEN");
    let hibp = hibp::HIBP::new(&client, hibp_token);

    let anonaddy_token_env = std::env::var("ANONADDY_TOKEN");
    let anonaddy_token = anonaddy_token_env.expect("Please provide ANONADDY_TOKEN");
    let anonaddy = anonaddy::AnonAddy::new(&client, anonaddy_token);

    let aliases = anonaddy.get_aliases().await?;
    for alias in aliases.data {
        if alias.active {
            println!("{:#?}", alias);
            let breaches = hibp.get_breaches().await?;
            if breaches.len() > 0 {
                println!("{:#?}", breaches);
            }
        }
    }
    Ok(())
}
