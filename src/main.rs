mod config;
mod error;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{config::Config, error::Error};

const ENDPOINT: &str = "https://api.porkbun.com/api/json/v3";
const ENDPOINT_IPV4: &str = "https://api-ipv4.porkbun.com/api/json/v3";

fn main() -> Result<(), Error> {
    let mut config = Config::read()?;
    if config.keys.is_none() {
        config.env_keys()?;
    }

    let agent = ureq::agent();

    let endpoint: &str = match config.ip.ipv6 {
        true => ENDPOINT,
        false => ENDPOINT_IPV4,
    };
    let ip = match config.ip.address.is_empty() {
        true => {
            let ping_response: Value = agent
                .post(&format!("{}/ping", endpoint))
                .send_json(config.try_keys())?
                .into_body()
                .read_json()?;
            if let Some(x) = ping_response["yourIp"].as_str() {
                x.to_owned()
            } else {
                return Err(Error::NoIp);
            }
        }
        false => config.ip.address.clone(),
    };

    let full_domain = match config.domain.subdomain.is_empty() {
        true => config.domain.base.clone(),
        false => {
            format!("{}.{}", config.domain.subdomain, &config.domain.base)
        }
    };

    let record_type = match config.ip.ipv6 {
        true => "AAAA",
        false => "A",
    };

    let mut ttl = None;
    let mut prio = None;
    let mut notes = None;

    let records_endpoint = format!(
        "{}/dns/retrieveByNameType/{}/{}/{}",
        endpoint, &config.domain.base, record_type, &config.domain.subdomain
    );
    let records_response: RecordsResponse = agent
        .post(&records_endpoint)
        .send_json(config.try_keys())?
        .into_body()
        .read_json()?;
    if records_response.status.as_str() != "SUCCESS" {
        return Err(Error::NoRecords);
    }

    let record = records_response.records.first();

    if let Some(x) = record {
        if x.content == ip {
            println!(
                "Existing {} record already matches answer {}",
                record_type, &ip
            );
            return Ok(());
        }
        let delete_endpoint = format!("{}/dns/delete/{}/{}", endpoint, &config.domain.base, x.id);
        let delete_response: Value = agent
            .post(&delete_endpoint)
            .send_json(config.try_keys())?
            .into_body()
            .read_json()?;
        if delete_response["status"]
            .as_str()
            .is_some_and(|x| x == "SUCCESS")
        {
            println!("Deleting existing {} record", &record_type);
        } else {
            return Err(Error::Delete);
        }
        ttl = x.ttl.clone();
        prio = x.prio.clone();
        notes = x.notes.clone();
    } else {
        println!("No record to be deleted.")
    }

    let create_endpoint = format!("{}/dns/create/{}", endpoint, &config.domain.base);
    let create_body = CreateRecord {
        secretapikey: config.try_keys().secretapikey.clone(),
        apikey: config.try_keys().apikey.clone(),
        name: config.domain.subdomain,
        _type: String::from(record_type),
        content: ip.to_string(),
        ttl,
        prio,
        notes,
    };
    let create_response: Value = agent
        .post(&create_endpoint)
        .send_json(create_body)?
        .into_body()
        .read_json()?;
    if create_response["status"]
        .as_str()
        .is_some_and(|x| x == "SUCCESS")
    {
        println!("Creating record: {} with answer of {}", &full_domain, &ip);
        Ok(())
    } else {
        Err(Error::Create)
    }
}

#[derive(Deserialize)]
struct RecordsResponse {
    status: String,
    records: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    id: String,
    #[serde(rename = "type")]
    _type: String,
    content: String,
    ttl: Option<String>,
    prio: Option<String>,
    notes: Option<String>,
}

#[derive(Serialize)]
struct CreateRecord {
    secretapikey: String,
    apikey: String,
    name: String,
    #[serde(rename = "type")]
    _type: String,
    content: String,
    ttl: Option<String>,
    prio: Option<String>,
    notes: Option<String>,
}
