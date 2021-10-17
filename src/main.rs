extern crate hyper;
extern crate hyper_rustls;
extern crate google_drive3 as drive3;
use drive3::Error;
use drive3::{DriveHub, oauth2};
use drive3::api::FileList;
use oauth2::{InstalledFlowAuthenticator, InstalledFlowReturnMethod};

#[tokio::main]
async fn main() {

    // See <https://docs.rs/yup-oauth2/6.0.0/yup_oauth2/>
    // TODO: Hunt for this file in the right places.
    let secret: oauth2::ApplicationSecret = yup_oauth2::read_application_secret("clientsecret.json")
        .await
        .expect("clientsecret.json");
    // TODO: Save this file in the right places.
    let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("tokencache.json")
        .build()
        .await
        .unwrap();

    let hub = DriveHub::new(hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()), auth);
    // You can configure optional parameters by calling the respective setters at will, and
    // execute the final call using `doit()`.
    // Values shown here are possibly random and not representative !
    let result = hub.files().list()
                 .supports_all_drives(true)
                 .spaces("drive")
                 .page_size(1000)
                 .order_by("modifiedTime desc")
                 .include_items_from_all_drives(true)
                 .doit().await;
     
    match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
             Error::HttpError(_)
            |Error::Io(_)
            |Error::MissingAPIKey
            |Error::MissingToken(_)
            |Error::Cancelled
            |Error::UploadSizeLimitExceeded(_, _)
            |Error::Failure(_)
            |Error::BadRequest(_)
            |Error::FieldClash(_)
            |Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok((_, file_list)) => print_files(file_list),
    }
}

fn print_files(res: FileList) {
    match res.files {
        Some(list) => {
            for file in list {
                if file.name.is_some() {
                    println!("{}", file.name.unwrap());
                }
            }
        },
        None => println!("No results")
    }
}
