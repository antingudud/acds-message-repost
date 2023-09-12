use serde::Deserialize;
use serde_json::Value;
use reqwest::Response;

use crate::Msg;
use std::error::Error;

#[derive(Debug)]
#[derive(Deserialize)]
pub struct Mastodon {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_code: String,
    pub token: String
}

impl Mastodon {
    pub async fn send(&self, message: &Msg) -> std::result::Result<Response, Box<dyn Error>> {
        let endpoint: &str = "https://mastodon.social/api/v1/statuses";
        let params = [
            ("status", &message.message),
            ("visibility", &String::from("public"))
        ];
        let client: reqwest::Client = reqwest::Client::new();
        let token: String;

        if &self.token == "" {
            token = self.get_token().await?;
        } else {
            token = self.token.clone();
        }

        let res = client.post(endpoint)
            .bearer_auth(token)
            .form(&params)
            .send()
            .await?;

        Ok(res)
    }

    async fn get_token(&self) -> Result<String, Box<dyn Error>> {
        let endpoint: &str = "https://mastodon.social/oauth/token";
        let params = [
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("redirect_uri", &self.redirect_uri),
            ("grant_type", &String::from("authorization_code")),
            ("code", &self.auth_code),
            ("scope", &String::from("write"))
        ];
        let client: reqwest::Client = reqwest::Client::new();

        println!("Auth code: {}", &self.auth_code);
        let res = client.post(endpoint)
            .form(&params)
            .send()
            .await?;

        if res.status().as_str() == "400" || res.status().as_str() == "401" {
            return Err(res.text().await?.into());
        }
        let res = res.text().await?;
        let data: Value = serde_json::from_str(res.as_str())?;
        let t = data["access_token"].as_str();
        let token: String = t.unwrap().to_string();
        println!("TOKEN COPY: {}", &token);
        Ok(token)
    }
}

pub mod message_pusher {
    use crate::Mastodon;
    use crate::Msg;
    use crate::Config;

    use reqwest::Response;
    use std::error::Error;

    pub struct MessagePusher {
        pub message: Msg,
        pub list: Mastodon
    }

    impl MessagePusher {
        pub fn build(msg: Msg, conf: Config) -> MessagePusher{
            MessagePusher {
                message: msg,
                list: conf.mastodon
            }
        }

        pub async fn push(&self) -> Result<String, Box<dyn Error>> {
            let msg = &self.message;
            let res: Response = self.list.send(msg).await?;

            Ok(res.text().await?)
        }
    }
}
