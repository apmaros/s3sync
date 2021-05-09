use google_cloud::authorize::ApplicationCredentials;
use google_cloud::storage;

fn load_creds() -> ApplicationCredentials {
    let creds = std::env::var("GCP_TEST_CREDENTIALS").expect("env GCP_TEST_CREDENTIALS not set");
    json::from_str::<ApplicationCredentials>(&creds)
        .expect("incorrect application credentials format")
}

pub(crate) async fn get_client() -> Result<storage::Client, storage::Error> {
    let creds = load_creds();
    storage::Client::from_credentials("main-313111", creds).await
}
