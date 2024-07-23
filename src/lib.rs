use std::{cell::RefCell, collections::HashMap};

use anyhow::{anyhow, bail};
use candid::{CandidType, Deserialize};
use ic_cdk::{query, update};
use ic_stable_structures::{BTreeMap as StableBTreeMap, DefaultMemoryImpl};
use serde::Serialize;
use serde_bytes::ByteBuf;
use serde_json::Value;
use url::Url;

thread_local! {
    static CONFIG_BY_VERSION: RefCell<StableBTreeMap<String, String, DefaultMemoryImpl>> = RefCell::new(
        StableBTreeMap::init(DefaultMemoryImpl::default())
    );
}

fn get_keys() -> Vec<String> {
    CONFIG_BY_VERSION.with(|p| p.borrow().iter().map(|(k, _v)| k).collect())
}

fn get_config(key: String) -> Option<String> {
    CONFIG_BY_VERSION.with(|p| p.borrow().get(&key))
}

fn put_config(key: String, value: String) {
    CONFIG_BY_VERSION.with(|p| p.borrow_mut().insert(key, value));
}

#[update]
fn add(version: String, config: String) -> String {
    let version = version.trim();
    match serde_json::from_str::<Value>(&config) {
        Ok(json) => {
            put_config(version.to_string(), serde_json::to_string(&json).unwrap());
            format!("Added config for version {}", version)
        }
        Err(err) => {
            format!("Invalid JSON: {}", err)
        }
    }
}

type HeaderField = (String, String);

#[derive(Debug, CandidType, Serialize)]
struct HttpResponse {
    status_code: u16,
    headers: Vec<HeaderField>,
    body: ByteBuf,
}

#[derive(Debug, CandidType, Deserialize)]
struct HttpRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: ByteBuf,
}

fn content_type(value: &str) -> HeaderField {
    ("Content-Type".to_string(), value.to_string())
}

fn http_request_impl(req: HttpRequest) -> Result<HttpResponse, anyhow::Error> {
    let url = Url::parse(&format!("https://localhost{}", req.url))?;
    match url.path() {
        "/versions" => Ok(HttpResponse {
            status_code: 200,
            headers: vec![content_type("application/json")],
            body: ByteBuf::from(serde_json::to_vec(&get_keys()).unwrap_or_default()),
        }),
        "/config" => {
            let query: HashMap<_, _> = url.query_pairs().into_owned().collect();
            let version = query
                .get("version")
                .ok_or(anyhow!("Required parameter is missing: version"))?;
            let config = get_config(version.clone())
                .ok_or(anyhow!("Config not found for version: {}", version))?;
            Ok(HttpResponse {
                status_code: 200,
                headers: vec![content_type("application/json")],
                body: ByteBuf::from(config.as_bytes()),
            })
        }
        _ => {
            bail!("Unsupported path: {}", url.path())
        }
    }
}

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    match http_request_impl(req) {
        Ok(res) => res,
        Err(err) => HttpResponse {
            status_code: 404,
            headers: vec![content_type("text/html")],
            body: ByteBuf::from(format!("Error: {:?}", err).as_bytes()),
        },
    }
}
