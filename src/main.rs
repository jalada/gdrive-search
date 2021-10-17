extern crate google_drive3 as drive3;
extern crate hyper;
extern crate hyper_rustls;
use async_recursion::async_recursion;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use drive3::Error;
use drive3::{oauth2, DriveHub};
use oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::error;
use std::fs::File;
use std::io::prelude::*;

const REFRESH_MINUTES: i64 = 30;

#[derive(Debug, Serialize, Deserialize)]
struct DriveFile {
    name: String,
    modified_time: DateTime<FixedOffset>,
}

impl DriveFile {
    fn from_drive3_file(file: drive3::api::File) -> DriveFile {
        // We assume all these fields exists, for now. Can probably add some
        // better error handling later (e.g. raise some exception and then
        // further down choose to ignore).
        DriveFile {
            name: file.name.unwrap(),
            modified_time: DateTime::parse_from_rfc3339(&file.modified_time.unwrap()).unwrap(),
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Missing argument");
        std::process::exit(1);
    }

    // TODO: Apparently you're a bad Rust person for using clone().
    // <https://doc.rust-lang.org/book/ch13-03-improving-our-io-project.html>
    let query = args[1].clone();

    // When did we last update the files?
    let last_fetched = get_last_fetched();
    let files: Vec<DriveFile> = if let Some(timestamp) = last_fetched {
        // Is it recent enough?
        if timestamp < Utc::now() - Duration::minutes(REFRESH_MINUTES) {
            print!("Cache of files is old. Fetching again ");
            let result = update_files().await.unwrap();
            println!("Done!");
            result
        } else {
            load_files_from_disk().unwrap_or_else(|_| panic!("Couldn't load files from disk"))
        }
    } else {
        println!("Never fetched files before, fetching again");
        update_files().await.unwrap()
    };

    // Time to hunt for a file!
    let results = find_files_matching(&query, files);

    println!("{:?}", results);
}

fn find_files_matching(query: &str, files: Vec<DriveFile>) -> Vec<DriveFile> {
    let mut results: Vec<DriveFile> = Vec::new();

    let needle = RegexBuilder::new(&regex::escape(query))
        .case_insensitive(true)
        .build()
        .expect("Bad Regex");

    for file in files {
        if needle.is_match(&file.name) {
            results.push(file);
        }
    }

    results
}

// https://doc.rust-lang.org/rust-by-example/error/multiple_error_types.html
// You could improve this by defining a custom error type. Boxing is a bit
// of a cheat and not really great 'cos it's dynamic.
fn load_files_from_disk() -> Result<Vec<DriveFile>, Box<dyn error::Error>> {
    let mut f = File::open("files.json")?;
    let mut buf = vec![];
    f.read_to_end(&mut buf)?;
    let files: Vec<DriveFile> = serde_json::from_slice(&buf)?;
    Ok(files)
}

fn get_last_fetched() -> Option<DateTime<Utc>> {
    if let Ok(mut f) = File::open("LAST_FETCHED") {
        let mut buf = vec![];
        if f.read_to_end(&mut buf).is_ok() {
            if let Ok(last_fetched) = serde_json::from_slice(&buf) {
                return last_fetched;
            }
        }
    }
    None
}

// TODO: This needs to accept our LAST_FETCHED timestamp so we can quickly
// fetch new/modified files. But then it'll also need to be smart about merging
// those files back in to our array. Quite a big change so skipping it for now
// whilst I work on other stuff
async fn update_files() -> std::io::Result<Vec<DriveFile>> {
    // See <https://docs.rs/yup-oauth2/6.0.0/yup_oauth2/>
    // TODO: Hunt for this file in the right places.
    let secret: oauth2::ApplicationSecret =
        yup_oauth2::read_application_secret("clientsecret.json")
            .await
            .expect("clientsecret.json");
    // TODO: Save this file in the right places.
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();

    let hub = DriveHub::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    );

    // Initialize empty file list
    let mut files: Vec<DriveFile> = Vec::new();

    // Go fetch
    files = fetch_files(hub, files, None).await;

    // Serialize
    let mut f = File::create("files.json")?;
    let buf = serde_json::to_vec(&files)?;
    f.write_all(&buf)?;

    // Record timestamp of this fetch
    let mut f = File::create("LAST_FETCHED")?;
    let buf = serde_json::to_vec(&Utc::now().to_rfc3339())?;
    f.write_all(&buf)?;
    Ok(files)
}

#[async_recursion]
async fn fetch_files(
    hub: DriveHub,
    mut files: Vec<DriveFile>,
    page_token: Option<String>,
) -> Vec<DriveFile> {
    print!(" . ");
    // <https://developers.google.com/drive/api/v3/reference/files/list>
    let mut result = hub
        .files()
        .list()
        .supports_all_drives(true)
        .spaces("drive")
        .page_size(1000)
        .include_items_from_all_drives(true)
        .param("fields", "nextPageToken, files(name, modifiedTime)");

    if let Some(token) = page_token {
        result = result.page_token(&token);
    }

    match result.doit().await {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::Io(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => panic!("Request failed when syncing file list"),
        },
        Ok((_, file_list)) => {
            // Assuming this is always a Vec...?
            for file in file_list.files.unwrap() {
                files.push(DriveFile::from_drive3_file(file))
            }
            match file_list.next_page_token {
                // Recurse
                Some(_) => fetch_files(hub, files, file_list.next_page_token).await,
                // Done
                None => files,
            }
        }
    }
}
