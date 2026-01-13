use aws_sdk_s3::Client;
use super::model::S3Error;

/// Create an S3 client using environment variables for credentials
pub async fn create_s3_client() -> Client {
    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    Client::new(&config)
}

/// Check if an atlas already exists in S3 cache
/// Returns the public URL if the object exists, None if not found
pub async fn check_cache(
    client: &Client,
    bucket: &str,
    key: &str,
) -> Result<Option<String>, S3Error> {
    match client.head_object().bucket(bucket).key(key).send().await {
        Ok(_) => {
            // Object exists, construct public URL
            let url = format!(
                "https://{}.s3.amazonaws.com/{}",
                bucket,
                key
            );
            Ok(Some(url))
        }
        Err(e) => {
            // Check if it's a 404 (not found) or actual error
            let error_msg = format!("{:?}", e);
            if error_msg.contains("NotFound") || error_msg.contains("404") {
                Ok(None)
            } else {
                Err(S3Error::UploadFailed(format!("Cache check failed: {}", error_msg)))
            }
        }
    }
}

/// Upload an atlas PNG to S3 with public-read ACL
/// Returns the public URL of the uploaded object
pub async fn upload_atlas(
    client: &Client,
    bucket: &str,
    key: &str,
    png_data: Vec<u8>,
) -> Result<String, S3Error> {
    let body = aws_sdk_s3::primitives::ByteStream::from(png_data);

    match client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .content_type("image/png")
        .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await
    {
        Ok(_) => {
            let url = format!(
                "https://{}.s3.amazonaws.com/{}",
                bucket,
                key
            );
            Ok(url)
        }
        Err(e) => Err(S3Error::UploadFailed(format!("Upload failed: {:?}", e))),
    }
}
