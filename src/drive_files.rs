extern crate google_drive3 as drive;
extern crate serde;
extern crate skim;

use async_recursion::async_recursion;
use chrono::{DateTime, FixedOffset, Utc};
use drive3::Error;
use drive3::{oauth2, DriveHub};
use oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use serde::{Deserialize, Serialize};
use skim::prelude::*;
use skim::SkimItem;
use std::error;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct DriveFile {
    id: String,
    name: String,
    modified_time: DateTime<FixedOffset>,
    web_view_link: String,
}

impl SkimItem for DriveFile {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }

    fn output(&self) -> Cow<str> {
        Cow::Borrowed(&self.web_view_link)
    }
}

impl DriveFile {
    pub fn from_drive3_file(file: drive3::api::File) -> DriveFile {
        // We assume all these fields exists, for now. Can probably add some
        // better error handling later (e.g. raise some exception and then
        // further down choose to ignore).
        DriveFile {
            id: file.id.unwrap(),
            web_view_link: file.web_view_link.unwrap(),
            name: file.name.unwrap(),
            modified_time: DateTime::parse_from_rfc3339(&file.modified_time.unwrap()).unwrap(),
        }
    }
}

pub struct DriveFiles {
    pub files: Vec<DriveFile>,
}

impl DriveFiles {
    pub fn new() -> DriveFiles {
        DriveFiles { files: Vec::new() }
    }

    fn contains(&self, f: &DriveFile) -> bool {
        // Look for a matching file with the same ID
        for file in &self.files {
            if file.id == f.id {
                return true;
            }
        }
        false
    }

    // https://doc.rust-lang.org/rust-by-example/error/multiple_error_types.html
    // You could improve this by defining a custom error type. Boxing is a bit
    // of a cheat and not really great 'cos it's dynamic.
    pub fn load_from_disk() -> Result<Self, Box<dyn error::Error>> {
        let mut f = File::open("files.json")?;
        let mut buf = vec![];
        f.read_to_end(&mut buf)?;
        let files: Vec<DriveFile> = serde_json::from_slice(&buf)?;
        Ok(Self { files })
    }

    pub fn last_fetched() -> Option<DateTime<Utc>> {
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

    pub async fn update_from_gdrive(
        mut self,
        modified_since: Option<DateTime<Utc>>,
    ) -> std::io::Result<Self> {
        // See <https://docs.rs/yup-oauth2/6.0.0/yup_oauth2/>
        // TODO: Hunt for this file in the right places.
        let secret: oauth2::ApplicationSecret =
            yup_oauth2::read_application_secret("clientsecret.json")
                .await
                .expect("clientsecret.json");
        // TODO: Save this file in the right places.
        let auth =
            InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
                .persist_tokens_to_disk("tokencache.json")
                .build()
                .await
                .unwrap();

        let hub = DriveHub::new(
            hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
            auth,
        );

        // Go fetch
        self.fetch_files(hub, modified_since, None).await;

        // Serialize
        let mut f = File::create("files.json")?;
        let buf = serde_json::to_vec(&self.files)?;
        f.write_all(&buf)?;

        // Record timestamp of this fetch
        let mut f = File::create("LAST_FETCHED")?;
        let buf = serde_json::to_vec(&Utc::now().to_rfc3339())?;
        f.write_all(&buf)?;
        Ok(self)
    }

    #[async_recursion]
    async fn fetch_files(
        &mut self,
        hub: DriveHub,
        modified_since: Option<DateTime<Utc>>,
        page_token: Option<String>,
    ) -> &Self {
        print!(" . ");
        std::io::stdout().flush().unwrap();
        // <https://developers.google.com/drive/api/v3/reference/files/list>
        let mut result = hub
            .files()
            .list()
            .supports_all_drives(true)
            .spaces("drive")
            .page_size(1000)
            .include_items_from_all_drives(true)
            // Order results most recent first so FZF prioritises those.
            .order_by("modifiedTime desc")
            .param(
                "fields",
                "nextPageToken, files(id, webViewLink, name, modifiedTime)",
            );

        if let Some(token) = page_token {
            result = result.page_token(&token);
        }

        if let Some(timestamp) = modified_since {
            result = result.q(&format!("modifiedTime > '{}'", timestamp.to_rfc3339()).to_string());
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
                | Error::JsonDecodeError(_, _) => panic!("{}", e),
            },
            Ok((_, file_list)) => {
                // Assuming this is always a Vec...?
                for file in file_list.files.unwrap() {
                    let f = DriveFile::from_drive3_file(file);
                    if !self.contains(&f) {
                        self.files.push(f);
                    }
                }
                match file_list.next_page_token {
                    // Recurse
                    Some(_) => {
                        self.fetch_files(hub, modified_since, file_list.next_page_token)
                            .await
                    }
                    // Done
                    None => self,
                }
            }
        }
    }
}
