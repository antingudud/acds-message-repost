use std::process;

use serenity::prelude::*;

use acds_bot::Config;
use acds_bot::Handler;

#[tokio::main]
async fn main() {
    let config = Config::build().unwrap_or_else(|err| {
        println!("Error building config: {}", err);
        process::exit(1);
    });
    let handler: Handler = Handler;

    let mut client = Client::builder(config.token, GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT).event_handler(handler).await.expect("Err creating client.");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    };
}
