use rusoto_s3::{S3Client, CreateBucketRequest, S3, PutObjectRequest, StreamingBody, GetObjectRequest, ListObjectsV2Request};
use crate::GenError;
use rusoto_core::{HttpClient, Client, Region};
use rusoto_credential::EnvironmentProvider;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct StoreClient {
    inner: S3Client
}

impl StoreClient {
    pub fn new() -> Result<StoreClient, GenError> {
        let dispatcher = HttpClient::new()?;

        let client = Client::new_with(EnvironmentProvider::default(), dispatcher);
        // todo parametrize region
        // todo set EUWEST1 for bucket
        let inner = S3Client::new_with_client(client, Region::UsEast1);

        Ok(StoreClient { inner })
    }

    #[allow(dead_code)]
    pub async fn create_bucket(&self, name: String) -> Result<String, GenError>{
        let request = CreateBucketRequest{
            acl: None,
            bucket: name,
            ..CreateBucketRequest::default()
        };

        let res = self.inner.create_bucket(request).await?;

        Ok(res.location.unwrap())
    }

    pub async fn put(&self, key: String, buffer: &[u8], bucket: String) -> Result<String, GenError>{
        let req = PutObjectRequest {
            key,
            body: Some(StreamingBody::from(Vec::from(buffer))),
            bucket,
            ..Default::default()
        };
        // todo  return response instead of etag only
        let res = self.inner.put_object(req).await?;
        Ok(res.e_tag.unwrap())
    }

    pub async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, GenError>{
        let req = GetObjectRequest {
            bucket: bucket.to_owned(),
            key: key.to_owned(),
            ..GetObjectRequest::default()
        };

        let res = self.inner.get_object(req).await?;

        let mut stream = res.body.unwrap().into_async_read();
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;

        Ok(buffer)
    }

    pub async fn list_keys(&self, bucket: &str, path: Option<&str>) -> Result<Vec<String>, GenError>{
        let req = ListObjectsV2Request {
            bucket: bucket.to_string(),
            prefix: path.map(String::from),
            ..ListObjectsV2Request::default()
        };

        let resp = self.inner
            .list_objects_v2(req.clone())
            .await?;

        let keys = resp.contents
            .unwrap_or(vec![])
            .into_iter()
            .map(|o| o.key)
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .collect();

        Ok(keys)
    }

    pub async fn list_folders(&self, bucket: String, path: Option<String>) -> Result<Vec<String>, GenError>{
        let req = ListObjectsV2Request {
            bucket: bucket.to_string(),
            delimiter: Some("/".to_owned()),
            prefix: path,
            ..ListObjectsV2Request::default()
        };

        let resp = self.inner
            .list_objects_v2(req.clone())
            .await?;

        let prefixes = resp.common_prefixes
            .unwrap_or(vec![])
            .into_iter()
            .map(|p| p.prefix)
            .filter(|p| p.is_some())
            .map(|p| p.unwrap())
            .collect();

        Ok(prefixes)
    }
}