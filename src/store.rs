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

pub(crate) async fn list_bucket_names() -> Result<Vec<String>, storage::Error> {
    let buckets = get_client().await?.buckets().await?;
    let bucket_names = buckets.iter()
        .map(|b| String::from(b.name()))
        .collect::<Vec<_>>();

    Ok(bucket_names)
}
