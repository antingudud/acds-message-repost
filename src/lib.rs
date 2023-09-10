use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

pub struct Config {
    pub token: String,
    pub intents: GatewayIntents
}

impl Config {
    pub fn build() -> Result<Config, &'static str> {
        let token: String = match env::var("DISCORD_TOKEN") {
            Ok(v) => v,
            Err(_) => return Err("Expected a token in the environment.")
        };
        let intents: GatewayIntents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

        let conf = Config {
            token,
            intents
        };
        Ok(conf)
    }
}

pub struct  Msg{
    pub message: String,
    pub attachment: String
}

impl Msg {
    pub fn new() -> Msg {
        Msg {
            message: String::from(""),
            attachment: String::from("")
        }
    }

    pub fn build_message(&self, msg: &Message) -> String {
        let mut message: String;
        let txt: Option<String> = self.get_content(&msg);
        let image: Option<String> = self.get_image(&msg);

        if txt.is_none() && image.is_some() {
            message = image.unwrap();
        } else {
            message = txt.unwrap();
            message.push_str("\n");
            message.push_str(
                &image.unwrap_or_else(|| {String::from("")})
            );
        }

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

        if msg.author.id == myself {
            println!("In: {}", msg.channel_id);
            println!("{}", msg.content);
            println!("\n");
        }        

        if msg.author.id == carl {
            println!("Embed (star): {:?}\n", msg.embeds);
            let desc: String = mesg.build_message(&msg);

            if let Err(why) = msg.channel_id.say(&ctx.http, desc).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected.", ready.user.name);
    }
}
