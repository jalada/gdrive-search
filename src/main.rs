extern crate async_recursion;
extern crate chrono;
extern crate google_drive3 as drive3;
extern crate hyper;
extern crate hyper_rustls;
extern crate webbrowser;
use chrono::{Duration, Utc};
use std::io::prelude::*;

mod drive_files;
use crate::drive_files::DriveFiles;

extern crate skim;
use skim::prelude::*;

const REFRESH_MINUTES: i64 = 1;

#[tokio::main]
async fn main() {
    // When did we last update the files?
    let last_fetched = DriveFiles::last_fetched();
    let files: DriveFiles = if let Some(timestamp) = last_fetched {
        // Is it recent enough?
        if timestamp < Utc::now() - Duration::minutes(REFRESH_MINUTES) {
            print!("Cache of files is old. Fetching again ");
            std::io::stdout()
                .flush()
                .unwrap();
            let result = DriveFiles::load_from_disk()
                .unwrap_or_else(|_| panic!("Couldn't load files from disk"))
                .update_from_gdrive(last_fetched)
                .await
                .unwrap();
            println!("Done!");
            result
        } else {
            DriveFiles::load_from_disk().unwrap_or_else(|_| panic!("Couldn't load files from disk"))
        }
    } else {
        println!("Never fetched files before, fetching");
        DriveFiles::new().update_from_gdrive(None).await.unwrap()
    };

    let options = SkimOptionsBuilder::default()
        .tiebreak(Some("index".to_string()))
        .build()
        .unwrap();

    // This is a bit hacky for now
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = unbounded();
    for file in files.files {
        let _ = tx_item.send(Arc::new(file));
    }
    drop(tx_item);

    let skim_output = Skim::run_with(&options, Some(rx_item));

    // This check then use feels un-Rusty
    if skim_output.as_ref().unwrap().is_abort {
        std::process::exit(1);
    }

    let selected_items = skim_output
        .map(|out| out.selected_items)
        .unwrap_or_else(Vec::new);

    for item in selected_items.iter() {
        // There will only be one.
        match webbrowser::open(&item.output()) {
            Ok(_) => {
                // Could be related to
                // https://github.com/amodm/webbrowser-rs/issues/18
                std::thread::sleep(std::time::Duration::from_millis(500));
                std::process::exit(0)
            }
            Err(_) => std::process::exit(1),
        }
    }
}
