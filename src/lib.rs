use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const BASE_URL: &str = "https://api.enrow.io";

// --- Request types ---

#[derive(Debug, Clone, Serialize)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FindPhoneParams {
    pub linkedin_url: Option<String>,
    pub fullname: Option<String>,
    pub company_domain: Option<String>,
    pub company_name: Option<String>,
    pub webhook: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BulkSearch {
    pub linkedin_url: Option<String>,
    pub fullname: Option<String>,
    pub company_domain: Option<String>,
    pub company_name: Option<String>,
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct FindPhonesParams {
    pub searches: Vec<BulkSearch>,
    pub webhook: Option<String>,
}

// --- Response types ---

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneResult {
    pub id: String,
    pub number: Option<String>,
    pub country: Option<String>,
    pub qualification: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub credits_used: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkPhoneResult {
    pub batch_id: String,
    pub total: u32,
    pub status: String,
    pub credits_used: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkPhoneResults {
    pub batch_id: String,
    pub status: String,
    pub total: u32,
    pub completed: Option<u32>,
    pub credits_used: Option<u32>,
    pub results: Option<Vec<PhoneResult>>,
}

// --- Internal helpers ---

#[derive(Debug, Deserialize)]
struct ApiError {
    message: Option<String>,
}

fn build_client(api_key: &str) -> Result<Client, Box<dyn std::error::Error>> {
    use reqwest::header::{HeaderMap, HeaderValue};

    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", HeaderValue::from_str(api_key)?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
}

async fn check_response(
    response: reqwest::Response,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let status = response.status();
    let body: serde_json::Value = response.json().await?;

    if !status.is_success() {
        let msg = body
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown API error");
        return Err(format!("API error {}: {}", status.as_u16(), msg).into());
    }

    Ok(body)
}

fn build_settings(
    webhook: &Option<String>,
) -> Option<Settings> {
    if webhook.is_some() {
        Some(Settings {
            webhook: webhook.clone(),
        })
    } else {
        None
    }
}

// --- Public API ---

/// Start a single phone search. Returns a `PhoneResult` containing the search ID.
/// Poll with `get_phone_result` to retrieve the final result, or provide a webhook URL.
pub async fn find_phone(
    api_key: &str,
    params: &FindPhoneParams,
) -> Result<PhoneResult, Box<dyn std::error::Error>> {
    let client = build_client(api_key)?;

    let mut body = serde_json::Map::new();

    if let Some(ref url) = params.linkedin_url {
        body.insert("linkedin_url".into(), serde_json::Value::String(url.clone()));
    }
    if let Some(ref name) = params.fullname {
        body.insert("fullname".into(), serde_json::Value::String(name.clone()));
    }
    if let Some(ref domain) = params.company_domain {
        body.insert("company_domain".into(), serde_json::Value::String(domain.clone()));
    }
    if let Some(ref name) = params.company_name {
        body.insert("company_name".into(), serde_json::Value::String(name.clone()));
    }

    if let Some(settings) = build_settings(&params.webhook) {
        body.insert("settings".into(), serde_json::to_value(settings)?);
    }

    let response = client
        .post(format!("{}/phone/single", BASE_URL))
        .json(&body)
        .send()
        .await?;

    let data = check_response(response).await?;
    let result: PhoneResult = serde_json::from_value(data)?;
    Ok(result)
}

/// Retrieve the result of a single phone search by its ID.
pub async fn get_phone_result(
    api_key: &str,
    id: &str,
) -> Result<PhoneResult, Box<dyn std::error::Error>> {
    let client = build_client(api_key)?;

    let response = client
        .get(format!("{}/phone/single?id={}", BASE_URL, id))
        .send()
        .await?;

    let data = check_response(response).await?;
    let result: PhoneResult = serde_json::from_value(data)?;
    Ok(result)
}

/// Start a bulk phone search. Returns a `BulkPhoneResult` with a batch ID.
/// Poll with `get_phone_results` to retrieve results, or provide a webhook URL.
pub async fn find_phones(
    api_key: &str,
    params: &FindPhonesParams,
) -> Result<BulkPhoneResult, Box<dyn std::error::Error>> {
    let client = build_client(api_key)?;

    let searches: Vec<serde_json::Value> = params
        .searches
        .iter()
        .map(|s| {
            let mut entry = serde_json::Map::new();

            if let Some(ref url) = s.linkedin_url {
                entry.insert("linkedin_url".into(), serde_json::Value::String(url.clone()));
            }
            if let Some(ref name) = s.fullname {
                entry.insert("fullname".into(), serde_json::Value::String(name.clone()));
            }
            if let Some(ref domain) = s.company_domain {
                entry.insert("company_domain".into(), serde_json::Value::String(domain.clone()));
            }
            if let Some(ref name) = s.company_name {
                entry.insert("company_name".into(), serde_json::Value::String(name.clone()));
            }
            if let Some(ref custom) = s.custom {
                entry.insert("custom".into(), serde_json::to_value(custom).unwrap());
            }

            serde_json::Value::Object(entry)
        })
        .collect();

    let mut body = serde_json::Map::new();
    body.insert("searches".into(), serde_json::Value::Array(searches));

    if let Some(settings) = build_settings(&params.webhook) {
        body.insert("settings".into(), serde_json::to_value(settings)?);
    }

    let response = client
        .post(format!("{}/phone/bulk", BASE_URL))
        .json(&body)
        .send()
        .await?;

    let data = check_response(response).await?;
    let result: BulkPhoneResult = serde_json::from_value(data)?;
    Ok(result)
}

/// Retrieve the results of a bulk phone search by its batch ID.
pub async fn get_phone_results(
    api_key: &str,
    id: &str,
) -> Result<BulkPhoneResults, Box<dyn std::error::Error>> {
    let client = build_client(api_key)?;

    let response = client
        .get(format!("{}/phone/bulk?id={}", BASE_URL, id))
        .send()
        .await?;

    let data = check_response(response).await?;
    let result: BulkPhoneResults = serde_json::from_value(data)?;
    Ok(result)
}
