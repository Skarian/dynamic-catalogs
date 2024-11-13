use anyhow::{anyhow, Context, Result};
use once_cell::sync::OnceCell;
use reqwest::Client;
use std::env;

pub fn set_globals() -> Result<()> {
    Environment::set()?;
    GlobalClient::set()?;
    Ok(())
}

#[derive(Debug)]
pub struct Environment {
    pub trakt_client_id: String,
    pub trakt_client_secret: String,
}

static ENVIRONMENT: OnceCell<Environment> = OnceCell::new();

impl Environment {
    pub fn set() -> Result<()> {
        let mut trakt_client_id = String::new();
        let mut trakt_client_secret = String::new();

        let environment = dotenvy::dotenv();

        match environment {
            Ok(_) => {
                for (key, value) in env::vars() {
                    match key.as_str() {
                        "TRAKT_CLIENT_ID" => trakt_client_id = value,
                        "TRAKT_CLIENT_SECRET" => trakt_client_secret = value,
                        _ => {}
                    }
                }
            }
            Err(_) => return Err(anyhow!("Unable to find requisite environment variables")),
        }

        let env = Self {
            trakt_client_id,
            trakt_client_secret,
        };

        // Check if any value is empty and return an error if so
        if env.trakt_client_id.is_empty() || env.trakt_client_secret.is_empty() {
            return Err(anyhow!(
                "1 or more required environment variables have 0 length"
            ));
        }

        ENVIRONMENT
            .set(env)
            .map_err(|_| anyhow!("Environment is already set"))?;
        Ok(())
    }
    pub fn get() -> Result<&'static Environment> {
        ENVIRONMENT
            .get()
            .context("Environment cell is empty, or being initialized")
    }
}

static CLIENT: OnceCell<Client> = OnceCell::new();

pub struct GlobalClient;

impl GlobalClient {
    pub fn set() -> Result<()> {
        let client = Client::new();
        CLIENT
            .set(client)
            .map_err(|_| anyhow!("Global Client is already set"))?;
        Ok(())
    }
    pub fn get() -> Result<&'static Client> {
        CLIENT
            .get()
            .context("Global Client cell is empty, or being initialized")
    }
}
