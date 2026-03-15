use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use aws_sdk_s3::Client;
use aws_sdk_s3::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_s3::operation::{
    delete_object::DeleteObjectError,
    get_object::GetObjectError,
    head_object::HeadObjectError,
    put_object::PutObjectError,
};
use bytes::Bytes;
use std::error::Error;
use std::env;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct S3HttpError {
    status: StatusCode,
    message: String,
}

impl S3HttpError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }
}

impl Display for S3HttpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for S3HttpError {}

impl IntoResponse for S3HttpError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

pub fn get_bucket_name() -> String {
    env::var("S3_BUCKET").unwrap_or_else(|_| "media.52poke.com".to_string())
}

pub async fn exist_s3(client: &Client, key: &str) -> Result<bool, S3HttpError> {
    let bucket = get_bucket_name();
    match client.head_object().bucket(&bucket).key(key).send().await {
        Ok(_) => Ok(true),
        Err(err) => {
            let http_error = s3_http_error_from_sdk(err);
            if http_error.status() == StatusCode::NOT_FOUND {
                Ok(false)
            } else {
                Err(http_error)
            }
        }
    }
}

pub async fn read_from_s3(client: &Client, key: &str) -> Result<(Bytes, String), S3HttpError> {
    let bucket = get_bucket_name();
    let resp = client
        .get_object()
        .bucket(&bucket)
        .key(key)
        .send()
        .await
        .map_err(s3_http_error_from_sdk)?;

    let content_type = resp.content_type().unwrap_or("application/octet-stream").to_string();
    let data = resp
        .body
        .collect()
        .await
        .map_err(|err| S3HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?
        .into_bytes();

    Ok((data, content_type))
}

pub async fn upload_to_s3(
    client: &Client,
    key: &str,
    data: Bytes,
    content_type: &str,
) -> Result<(), S3HttpError> {
    let bucket = get_bucket_name();
    client.put_object()
        .bucket(&bucket)
        .key(key)
        .body(data.into())
        .content_type(content_type)
        .send()
        .await
        .map_err(s3_http_error_from_sdk)?;
    Ok(())
}

pub async fn delete_s3(client: &Client, key: &str) -> Result<(), S3HttpError> {
    let bucket = get_bucket_name();
    client
        .delete_object()
        .bucket(&bucket)
        .key(key)
        .send()
        .await
        .map_err(s3_http_error_from_sdk)?;
    Ok(())
}

fn s3_http_error_from_sdk<E>(err: SdkError<E>) -> S3HttpError
where
    E: Error + ProvideErrorMetadata + Send + Sync + 'static,
{
    let status = err
        .raw_response()
        .map(|raw| raw.status().as_u16())
        .and_then(|status| StatusCode::from_u16(status).ok())
        .map(normalize_status)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let message = err
        .as_service_error()
        .and_then(|service_error| service_error.message())
        .filter(|message| !message.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| err.to_string());

    S3HttpError::new(status, message)
}

fn normalize_status(status: StatusCode) -> StatusCode {
    match status {
        StatusCode::FORBIDDEN | StatusCode::NOT_FOUND => StatusCode::NOT_FOUND,
        other => other,
    }
}

#[allow(dead_code)]
fn _assert_error_traits() {
    fn assert_error<E: Error + ProvideErrorMetadata + Send + Sync + 'static>() {}
    assert_error::<HeadObjectError>();
    assert_error::<GetObjectError>();
    assert_error::<PutObjectError>();
    assert_error::<DeleteObjectError>();
}
