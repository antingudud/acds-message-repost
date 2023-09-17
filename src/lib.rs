use std::fs;
use std::error::Error;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use serde::Deserialize;

mod message_pusher;
use message_pusher::Mastodon;
use message_pusher::message_pusher::MessagePusher;

#[derive(Debug)]
#[derive(Deserialize)]
pub struct Config {
    pub token: String,
    pub mastodon: Mastodon
}

impl Config {
    pub fn build() -> std::result::Result<Config, Box<dyn Error>> {
        let config_path: &str = match env::var("TEST") {
            Ok(_) => "config.example.json",
            Err(_) => "config.json"
        };

        let config_file = fs::read_to_string(config_path)?;
        let conf: Config = serde_json::from_str(&config_file)?;

        Ok(conf)
    }
}

pub struct  Msg{
    pub message: Option<String>,
    pub attachment: Option<String>
}

impl Msg {
    pub fn new() -> Msg {
        Msg {
            message: None,
            attachment: None
        }
    }

    pub fn build_message(&self, msg: &Message) -> Msg {
        let txt: Option<String> = self.get_content(&msg);
        let image: Option<String> = self.get_image(&msg);

        let message: Msg = Msg {
            message: txt,
            attachment: image
        };

        message
    }

    pub fn get_content(&self, msg: &Message) -> Option<String> {
        println!("Embed (star): {:?}\n", msg.embeds);
        let desc: Option<String> = msg.embeds[0].description.clone();

        let desc = match desc {
            Some(v) => v,
            None => return None
        };

        Some(desc)
    }

    pub fn get_image(&self, msg: &Message) -> Option<String> {
        let url = match msg.embeds[0].image.clone() {
            Some(v) => v.url,
            None => return None
        };

        Some(url)
    }
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let myself: u64 = 1149630157913608246;
        let carl: u64 = 235148962103951360;
        let mesg: Msg = Msg::new();
        let conf: Config = Config::build().unwrap();

        if msg.author.id == myself {
            println!("In: {}", msg.channel_id);
            println!("{}", msg.content);
            println!("\n");
        }

        if msg.author.id == carl {
            println!("Embed (star): {:?}\n", msg.embeds);
            let desc: Msg = mesg.build_message(&msg);
            let pusher: MessagePusher = MessagePusher::build(desc, conf);

            let response: String = pusher.push().await.unwrap_or_else(|e| {e.to_string() + " WAS ERR"} );

            if let Err(why) = msg.channel_id.say(&ctx.http, response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_ccnfig_build() {
        let discord_token = "your.token";
        let config: Config = Config::build().unwrap();

        assert_eq!(discord_token, config.token);
        assert_eq!("client.id", config.mastodon.client_id);
        assert_eq!("client.secret", config.mastodon.client_secret);
        assert_eq!("uri", config.mastodon.redirect_uri);
        assert_eq!("jumbledtext", config.mastodon.auth_code);
        assert_eq!("your_token", config.mastodon.token);
    }
}
