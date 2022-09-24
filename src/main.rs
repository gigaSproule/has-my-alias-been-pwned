use dotenv::dotenv;
use log::{debug, error, info, warn, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};

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
    configure_logging();

    let env_file = dotenv();
    if env_file.is_err() {
        error!("Unable to find .env file");
    }
    let client = reqwest::Client::new();

    let alias_service = get_alias_service(&client);

    let hibp = hibp::HIBP::new(&client);

    let aliases = alias_service.get_aliases().await?;
    for alias in aliases {
        if alias.is_active() {
            info!(
                "Checking breaches for {} - {}",
                alias.get_email(),
                alias.get_description().unwrap_or("")
            );
            let breaches = hibp.get_breaches(alias.get_email()).await?;
            if !breaches.is_empty() {
                debug!("{:#?}", breaches);
                warn!(
                    "{} breaches were found for {} - {}",
                    breaches.len(),
                    alias.get_email(),
                    alias.get_description().unwrap_or("")
                );
                alias_service.deactivate_alias(alias.get_id()).await?;
            }
        }
    }
    Ok(())
}

fn configure_logging() {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}
