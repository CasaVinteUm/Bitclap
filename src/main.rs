use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_derive::Deserialize;

const PROJECT_NAME: &str = "bitdevs-assistant";
const ACCEPT_VALUE: &str = "application/vnd.github+json";
const API_URL: &str = "https://api.github.com";
const ISSUE_NUMBER: u32 = 12;
const ORG_NAME: &str = "lorenzolfm";
const REPO_NAME: &str = "floripabitdevs";

#[derive(Deserialize)]
pub struct GetIssueResponse {
    pub comments_url: String,
}

fn main() -> Result<(), reqwest::Error> {
    let github_token =
        std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable not set");

    let client = Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(USER_AGENT, HeaderValue::from_str(PROJECT_NAME).unwrap());
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&github_token).unwrap());
    headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_VALUE));

    let response = client
        .get(format!(
            "{API_URL}/repos/{ORG_NAME}/{REPO_NAME}/issues/{ISSUE_NUMBER}"
        ))
        .headers(headers)
        .send()?;

    println!("Status: {}", response.status());
    let body = response.text()?;
    println!("Body:\n{}", body);

    Ok(())
}
