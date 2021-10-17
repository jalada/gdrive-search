extern crate google_drive3 as drive3;
extern crate hyper;
extern crate hyper_rustls;
use async_recursion::async_recursion;
use chrono::{DateTime, FixedOffset};
use drive3::Error;
use drive3::{oauth2, DriveHub};
use oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

#[derive(Debug)]
struct File {
    name: String,
    modified_time: DateTime<FixedOffset>,
}

impl File {
    fn from_drive3_file(file: drive3::api::File) -> File {
        // We assume all these fields exists, for now.
        File {
            name: file.name.unwrap(),
            // TODO: This is failing to parse
            modified_time: DateTime::parse_from_rfc2822(&file.modified_time.unwrap()).unwrap(),
        }
    }
}

#[tokio::main]
async fn main() {
    // Update registry of files
    update_files().await;
}

async fn update_files() {
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
    let mut files: Vec<File> = Vec::new();

    files = fetch_files(hub, files, None).await;

    println!("{:?}", files);
}

#[async_recursion]
async fn fetch_files(hub: DriveHub, mut files: Vec<File>, page_token: Option<String>) -> Vec<File> {
    println!("Fetching page with token {:?}", page_token);
    // <https://developers.google.com/drive/api/v3/reference/files/list>
    let mut result = hub
        .files()
        .list()
        .supports_all_drives(true)
        .spaces("drive")
        .page_size(1000)
        .include_items_from_all_drives(true)
        .param("fields", "files(name, modifiedTime)");

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
                files.push(File::from_drive3_file(file))
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
