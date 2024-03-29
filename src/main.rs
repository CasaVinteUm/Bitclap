use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_derive::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;

const PROJECT_NAME: &str = "bitdevs-assistant";
const ACCEPT_VALUE: &str = "application/vnd.github+json";
const API_URL: &str = "https://api.github.com";
const ISSUE_NUMBER: u32 = 12;
const ORG_NAME: &str = "lorenzolfm";
const REPO_NAME: &str = "floripabitdevs";

#[derive(Deserialize, Debug)]
pub struct GetIssueResponse {
    pub comments_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    pub body: String,
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
        .headers(headers.clone())
        .send()?;

    let res: GetIssueResponse = response.json()?;

    let comments = client.get(res.comments_url).headers(headers).send()?;
    let comments: Vec<Comment> = comments.json()?;

    for comment in &comments {
        let Some((title, url)) = comment.body.split_once("\r\n") else {
            continue;
        };

        println!("{title}, {url}");
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("post.md")
        .expect("Unable to open file");

    for comment in &comments {
        if let Some((title, url)) = comment.body.split_once("\r\n") {
            writeln!(file, "* [{}]({})", title.trim(), url.trim())
                .expect("Unable to write to file");
        }
    }

    Ok(())
}
