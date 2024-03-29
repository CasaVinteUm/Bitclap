use clap::Parser;
use dotenv::dotenv;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde_derive::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;

const ACCEPT_VALUE: &str = "application/vnd.github+json";
const API_URL: &str = "https://api.github.com";

#[derive(Deserialize, Debug)]
pub struct GetIssueResponse {
    pub comments_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    pub body: String,
}

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

fn main() -> Result<(), reqwest::Error> {
    let Env {
        meetup_name,
        meetup_chat_link,
        gh_api_project_name,
        gh_api_token,
        repo_org,
        repo_name,
    } = Env::new();

    let Args {
        meetup_number,
        meetup_date,
        meetup_link,
        issue_number,
    } = Args::parse();

    let post_prefix = format!(
        "---\nlayout: post\ntype: socratic\ntitle: \"Seminário Socrático {}\"\nmeetup: {}\n---\n\n\
        ## Avisos\n\n\
        - Entrem no grupo do Whatsapp [{meetup_name}]({meetup_chat_link})!\n\
        - Respeite a privacidade dos participantes.\n\
        - Os meetups nunca são gravados. Queremos todos a vontade para participar e discutir os assuntos programados, de forma anônima se assim o desejarem.\n\n\
        ## Agradecimentos\n\n\
        - Agradecemos à Vinteum pela casa, comidas e bebidas.\n\n\
        ## Cronograma\n",
        meetup_number, meetup_link
    );

    let client = Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&gh_api_project_name).unwrap(),
    );
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&gh_api_token).unwrap());
    headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_VALUE));

    let response = client
        .get(format!(
            "{API_URL}/repos/{repo_org}/{repo_name}/issues/{issue_number}",
        ))
        .headers(headers.clone())
        .send()?;

    let res: GetIssueResponse = response.json()?;

    let comments = client.get(res.comments_url).headers(headers).send()?;
    let comments: Vec<Comment> = comments.json()?;

    let file_name = format!("{meetup_date}-socratic-seminar-{meetup_number}.md");

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_name)
        .expect("Unable to open file");

    writeln!(file, "{}", post_prefix).expect("Unable to write to file");

    for comment in &comments {
        if let Some((title, url)) = comment.body.split_once("\r\n") {
            writeln!(file, "* [{}]({})", title.trim(), url.trim())
                .expect("Unable to write to file");
        }
    }

    Ok(())
}
