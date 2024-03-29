use clap::Parser;
use dotenv::dotenv;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_derive::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    meetup_number: u16,
    #[arg(long)]
    meetup_date: String,
    #[arg(long)]
    meetup_link: String,

    #[arg(long)]
    issue_number: u16,
}

#[derive(Clone)]
struct Env {
    pub meetup_name: String,
    pub meetup_chat_link: String,

    pub gh_api_project_name: String,
    pub gh_api_token: String,

    pub repo_org: String,
    pub repo_name: String,
}

impl Env {
    fn new() -> Env {
        dotenv().ok();

        Env {
            meetup_name: std::env::var("MEETUP_NAME")
                .expect("MEETUP_NAME environment variable not set"),
            meetup_chat_link: std::env::var("MEETUP_CHAT_LINK")
                .expect("MEETUP_CHAT_LINK environment variable not set"),
            gh_api_project_name: std::env::var("GH_API_PROJECT_NAME")
                .expect("GH_API_PROJECT_NAME environment variable not set"),
            gh_api_token: std::env::var("GH_API_TOKEN")
                .expect("GH_API_TOKEN environment variable not set"),
            repo_org: std::env::var("REPO_ORG").expect("REPO_ORG environment variable not set"),
            repo_name: std::env::var("REPO_NAME").expect("REPO_NAME environment variable not set"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GetIssueResponse {
    pub comments_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    pub body: String,
}

#[derive(thiserror::Error, Debug)]
pub enum GetCommentsError {
    #[error("Reqwest: {0:?}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Invalid header: {0:?}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Github is not returning a 200 status code")]
    NotOk,
}

fn get_comments(env: Env, issue_number: u16) -> Result<Vec<Comment>, GetCommentsError> {
    const API_URL: &str = "https://api.github.com";
    const ACCEPT_VALUE: &str = "application/vnd.github+json";

    let gh_api_project_name = HeaderValue::from_str(&env.gh_api_project_name)?;
    let gh_api_token = HeaderValue::from_str(&env.gh_api_token)?;

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, gh_api_project_name);
    headers.insert(AUTHORIZATION, gh_api_token);
    headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_VALUE));

    let url = format!(
        "{API_URL}/repos/{}/{}/issues/{issue_number}",
        env.repo_org, env.repo_name,
    );

    let client = Client::new();

    let response = client.get(url).headers(headers.clone()).send()?;

    if response.status() != 200 {
        return Err(GetCommentsError::NotOk);
    }

    let res = response.json::<GetIssueResponse>()?;
    let comments = client.get(res.comments_url).headers(headers).send()?;
    let comments = comments.json::<Vec<Comment>>()?;

    Ok(comments)
}

struct WritePostMarkdownParams {
    meetup_date: String,
    meetup_number: u16,
    meetup_link: String,
    comments: Vec<Comment>,
}

fn write_post_markdown(env: Env, p: WritePostMarkdownParams) -> Result<(), std::io::Error> {
    let post_prefix = format!(
        "---\nlayout: post\ntype: socratic\ntitle: \"Seminário Socrático {}\"\nmeetup: {}\n---\n\n\
        ## Avisos\n\n\
        - Entrem no grupo do Whatsapp [{}]({})!\n\
        - Respeite a privacidade dos participantes.\n\
        - Os meetups nunca são gravados. Queremos todos a vontade para participar e discutir os assuntos programados, de forma anônima se assim o desejarem.\n\n\
        ## Agradecimentos\n\n\
        - Agradecemos à Vinteum pela casa, comidas e bebidas.\n\n\
        ## Cronograma\n",
        p.meetup_number, p.meetup_link, env.meetup_name, env.meetup_chat_link
    );

    let file_name = format!("{}-socratic-seminar-{}.md", p.meetup_date, p.meetup_number);

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file_name)?;

    writeln!(file, "{}", post_prefix)?;

    for comment in &p.comments {
        if let Some((title, url)) = comment.body.split_once("\r\n") {
            writeln!(file, "* [{}]({})", title.trim(), url.trim())?;
        }
    }

    Ok(())
}

fn main() -> Result<(), ()> {
    let env = Env::new();

    let Args {
        meetup_number,
        meetup_date,
        meetup_link,
        issue_number,
    } = Args::parse();

    let comments = get_comments(env.clone(), issue_number).map_err(|e| {
        eprintln!("Failed to get comments: {}", e);
    })?;

    write_post_markdown(
        env,
        WritePostMarkdownParams {
            meetup_date,
            meetup_number,
            meetup_link,
            comments,
        },
    )
    .map_err(|e| {
        eprintln!("Failed to write md file: {}", e);
    })?;

    Ok(())
}
