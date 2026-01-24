use std::error::Error;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use base64::prelude::*;

use reqwest::blocking::Client;
use reqwest::header::{USER_AGENT, AUTHORIZATION};

use crate::{config, store, funcs};
use crate::utils::crypto;

pub struct Github {
    token: String,
    owner: String,
    repo: String,
    client: Client
}

impl Github {
    pub fn new() -> Self {
        let token = match crypto::decrypt(config::GITHUB_API_KEY, config::CRYPTO_KEY) {
            Ok(token) => String::from_utf8_lossy(&token).into_owned(),
            Err(_) => process::exit(0) // cmd func shutdown
        };

        let owner = match crypto::decrypt(config::GITHUB_OWNER, config::CRYPTO_KEY) {
            Ok(owner) => String::from_utf8_lossy(&owner).into_owned(),
            Err(_) => process::exit(0) // cmd func shutdown
        };

        let repo = match crypto::decrypt(config::GITHUB_REPO, config::CRYPTO_KEY) {
            Ok(repo) => String::from_utf8_lossy(&repo).into_owned(),
            Err(_) => process::exit(0) // cmd func shutdown
        };

        let client = Client::new();

        Self { token, owner, repo, client }
    }

    fn get_url(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/contents",
            self.owner,
            self.repo
        )
    }

    fn get_auth(&self) -> String {
        format!("Bearer {}", self.token)
    }

    fn get_file_hash(&self, file: &str) -> Result<Option<String>, Box<dyn Error>> {
        let url = format!("{}/{}", self.get_url(), file);

        let res = self.client
            .get(&url)
            .header(AUTHORIZATION, self.get_auth())
            .header(USER_AGENT, store::USER_AGENT)
            .send()?;

        if res.status() == 404 {
            return Ok(None);
        }

        let json = res.json::<Value>()?;
        let hash = json["sha"]
            .as_str()
            .map(String::from);

        Ok(hash)
    }

    pub fn read_file(&self, file: &str) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/{}", self.get_url(), file);

        let res = self.client
            .get(&url)
            .header(AUTHORIZATION, self.get_auth())
            .header(USER_AGENT, store::USER_AGENT)
            .send()?;

        let json = res.json::<Value>()?;

        let content = json["content"]
            .as_str()
            .ok_or("failed to get content")?
            .replace("\n", "");

        let decoded = BASE64_STANDARD.decode(content)?;
        let stringed = String::from_utf8(decoded)?;

        Ok(stringed)
    }

    pub fn write_file(&self, file: &str, content: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/{}", self.get_url(), file);

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let body = serde_json::json!({
            "message": time,
            "content": BASE64_STANDARD.encode(content),
            "sha": self.get_file_hash(file).ok()
        });

        self.client
            .put(&url)
            .header(AUTHORIZATION, self.get_auth())
            .header(USER_AGENT, store::USER_AGENT)
            .json(&body)
            .send()?;

        Ok(())
    }

    pub fn delete_file(&self, file: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/{}", self.get_url(), file);

        let hash = self.get_file_hash(file)?
            .ok_or("failed to find file")?;

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        let body = serde_json::json!({
            "message": time,
            "sha": hash
        });

        self.client
            .delete(&url)
            .header(AUTHORIZATION, self.get_auth())
            .header(USER_AGENT, store::USER_AGENT)
            .json(&body)
            .send()?;

        Ok(())
    }
}