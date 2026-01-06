use aws_sdk_s3::Client;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::head_object::HeadObjectError;
use bytes::Bytes;
use std::env;

pub fn get_bucket_name() -> String {
    env::var("S3_BUCKET").unwrap_or_else(|_| "media.52poke.com".to_string())
}

pub async fn exist_s3(client: &Client, key: &str) -> bool {
    let bucket = get_bucket_name();
    match client.head_object().bucket(&bucket).key(key).send().await {
        Ok(_) => true,
        Err(SdkError::ServiceError(err)) => {
            match err.err() {
                HeadObjectError::NotFound(_) => false,
                // AccessDenied or 403 might also mean not found in some configs, or actually forbidden.
                // The original code treated Forbidden/403 as false (not found/accessible).
                _ => {
                    // Check status code if possible, or assume false for 404/403-like errors
                     let code = err.raw().status().as_u16();
                     if code == 404 || code == 403 {
                         false
                     } else {
                         false // Defaulting to false on error to be safe, or should we log?
                     }
                }
            }
        }
        Err(_) => false,
    }
}

pub async fn read_from_s3(client: &Client, key: &str) -> Result<(Bytes, String), anyhow::Error> {
    let bucket = get_bucket_name();
    let resp = client.get_object().bucket(&bucket).key(key).send().await?;
    
    let content_type = resp.content_type().unwrap_or("application/octet-stream").to_string();
    let data = resp.body.collect().await?.into_bytes();
    
    Ok((data, content_type))
}

pub async fn upload_to_s3(client: &Client, key: &str, data: Bytes, content_type: &str) -> Result<(), anyhow::Error> {
    let bucket = get_bucket_name();
    client.put_object()
        .bucket(&bucket)
        .key(key)
        .body(data.into())
        .content_type(content_type)
        .send()
        .await?;
    Ok(())
}

pub async fn delete_s3(client: &Client, key: &str) -> Result<(), anyhow::Error> {
    let bucket = get_bucket_name();
    client.delete_object().bucket(&bucket).key(key).send().await?;
    Ok(())
}
