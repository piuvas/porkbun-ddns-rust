use std::{
    env::{self, VarError},
    fs,
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub keys: Option<Keys>,
    pub domain: Domain,
    pub ip: Ip,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Keys {
    pub secretapikey: String,
    pub apikey: String,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Domain {
    pub subdomain: String,
    pub base: String,
}
#[derive(Serialize, Deserialize, Default)]
pub struct Ip {
    pub address: String,
    pub ipv6: bool,
}

impl Config {
    pub fn read() -> Result<Self, Error> {
        let config_path = Path::new("config.toml");
        match config_path.exists() {
            true => {
                let data = fs::read_to_string(config_path)?;
                Ok(toml::from_str(&data)?)
            }
            false => {
                let data = toml::to_string_pretty(&Config::default())?;
                fs::write(config_path, data)?;
                Err(Error::NoConfig(config_path.to_path_buf()))
            }
        }
    }
    pub fn env_keys(&mut self) -> Result<(), VarError> {
        self.keys = Some(Keys {
            secretapikey: env::var("PORKBUN_SECRET_API_KEY")?,
            apikey: env::var("PORKBUN_API_KEY")?,
        });
        Ok(())
    }
    pub(crate) fn try_keys(&self) -> &Keys {
        if let Some(keys) = &self.keys {
            keys
        } else {
            unreachable!();
        }
    }
}
