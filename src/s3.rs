use aws_sdk_s3::error::{ProvideErrorMetadata, SdkError};
use aws_sdk_s3::operation::{
    delete_object::DeleteObjectError, get_object::GetObjectError, head_object::HeadObjectError,
    put_object::PutObjectError,
};
use aws_sdk_s3::Client;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use std::env;
use std::error::Error;
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
            if matches!(
                http_error.status(),
                StatusCode::FORBIDDEN | StatusCode::NOT_FOUND
            ) {
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

    let content_type = resp
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();
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
    client
        .put_object()
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
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let message = err
        .as_service_error()
        .and_then(|service_error| service_error.message())
        .filter(|message| !message.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| err.to_string());

    S3HttpError::new(status, message)
}

#[allow(dead_code)]
fn _assert_error_traits() {
    fn assert_error<E: Error + ProvideErrorMetadata + Send + Sync + 'static>() {}
    assert_error::<HeadObjectError>();
    assert_error::<GetObjectError>();
    assert_error::<PutObjectError>();
    assert_error::<DeleteObjectError>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn client_responding_with(status: StatusCode) -> Client {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];

            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let bytes_read = socket.read(&mut buffer).await.unwrap();
                if bytes_read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..bytes_read]);
            }

            let response = format!(
                "HTTP/1.1 {} {}\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
                status.as_u16(),
                status.canonical_reason().unwrap()
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        });

        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(Credentials::new("test", "test", None, None, "test"))
            .region(Region::new("test-region-1"))
            .endpoint_url(format!("http://{address}"))
            .force_path_style(true)
            .build();

        Client::from_conf(config)
    }

    #[test]
    fn s3_http_error_response_preserves_status() {
        let response = S3HttpError::new(StatusCode::FORBIDDEN, "access denied").into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn s3_http_error_uses_its_message() {
        let error = S3HttpError::new(StatusCode::BAD_GATEWAY, "storage failure");

        assert_eq!(error.to_string(), "storage failure");
    }

    #[tokio::test]
    async fn existence_check_treats_forbidden_as_missing() {
        let client = client_responding_with(StatusCode::FORBIDDEN).await;

        assert!(!exist_s3(&client, "wiki/missing.png").await.unwrap());
    }

    #[tokio::test]
    async fn upload_preserves_forbidden_status() {
        let client = client_responding_with(StatusCode::FORBIDDEN).await;

        let error = upload_to_s3(
            &client,
            "wiki/thumb/a/ab/image.png/200px-image.png",
            Bytes::from_static(b"webp"),
            "image/webp",
        )
        .await
        .unwrap_err();

        assert_eq!(error.status(), StatusCode::FORBIDDEN);
    }
}
