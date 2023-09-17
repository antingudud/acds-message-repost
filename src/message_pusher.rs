use serde::{Serialize, Deserialize};
use serde_json::Value;
use reqwest::multipart::Part;

use crate::Msg;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct RequestParameter<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_ids: Option<Vec<String>>,
    visibility: &'a str
}

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
    pub async fn send(&self, message: &Msg) -> std::result::Result<String, Box<dyn Error>> {
        let endpoint: &str = "https://mastodon.social/api/v1/statuses";
        let media_id: Option<Vec<String>> = match &message.attachment {
            Some(v) => {
                let id = self.upload_image(v).await.unwrap_or_else(|e| {
                    // Unacceptable. This will result in a runtime error. Should fix it soon
                    println!("[message_pusher:send:media_id] ERROR: {e}"); String::from("None")
                });
                Some(vec!(id))
            }
            None => None
        };
        let request = RequestParameter {
            status: message.message.clone(),
            media_ids: media_id,
            visibility: "public"
        };
        let client: reqwest::Client = reqwest::Client::new();
        let token: String;

        if &self.token == "" {
            token = self.get_token().await?;
        } else {
            token = self.token.clone();
        }

        let j = serde_json::to_string(&request);
        println!("[message_pusher:send] INFO: request: {:?}", &j);
        let res = client.post(endpoint)
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        let t = res.text().await?;
        let v: Value = serde_json::from_str(t.as_str())?;
        let o = v["url"].as_str();
        let url: String = o.unwrap_or_else(|| {
            println!("[message_pusher:send] INFO: {}", t);
            "Error: somethng happened"
        } ).to_string();
        Ok(url)
    }

    async fn upload_image(&self, img_url: &String) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let res = client.get(img_url)
            .send()
            .await?
            .bytes()
            .await?;

        use std::fs::File;
        use std::io::Write;

        let mut file = File::create("image_log").expect("Creation failed");
        file.write(&res).expect("Writing failed");
        println!("[message_pusher:upload_image] INFO: Filed image get request");

        let form = reqwest::multipart::Form::new();
        let byte: Vec<u8> = Vec::from(res);
        let mut file = File::create("byte_log").expect("Creation failed");
        file.write(&byte).expect("Writing failed");
        println!("[message_pusher:upload_image] INFO: Filed image bytes");
        let part = Part::bytes(byte).file_name("image.png").mime_str("image/png");
        let form = form.part("file", part.unwrap());

        let med = client.post("https://mastodon.social/api/v2/media")
            .bearer_auth(&self.token)
            .multipart(form)
            .send()
            .await?;

        if med.status().as_str() == "200" {
            let d = med.text().await?;
            println!("[message_pusher:upload_image] INFO: MediaAttachment response\n {d}");
            let v: Value = serde_json::from_str(d.as_str())?;
            let j = v["id"].as_str();
            let id: String = j.unwrap().to_string();
            println!("[message_pusher:upload_image] INFO: media id is {}", &id);
            return Ok(id);
        } else {
            println!(
                "[message_pusher:upload_image] ERROR: Response status code is not 200\nStatus: {}\nContent Length: {}\nResponse: {}",
                med.status().as_str(),
                med.content_length().unwrap_or_else(|| 0),
                med.text().await.unwrap_or_else(|e| e.to_string()));
            return Err(String::from("4xx").into());
        }
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
            let res: String = self.list.send(msg).await?;

            Ok(res)
        }
    }
}
