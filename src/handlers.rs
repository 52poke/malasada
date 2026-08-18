use aws_sdk_s3::Client;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::{image_ops, s3, utils};

pub struct AppState {
    pub s3_client: Client,
}

pub async fn handle_resize(
    State(state): State<Arc<AppState>>,
    Path(path_param): Path<String>,
) -> impl IntoResponse {
    let full_path = format!("/wiki/thumb/{}", path_param);

    handle_thumb_path(&state.s3_client, &full_path).await
}

pub async fn handle_webp(
    State(state): State<Arc<AppState>>,
    Path(path_param): Path<String>,
) -> impl IntoResponse {
    let key = path_param;

    if key.starts_with("wiki/thumb/") {
        return handle_thumb_path(&state.s3_client, &format!("/{}", key)).await;
    }

    if !key.to_lowercase().ends_with(".jpg")
        && !key.to_lowercase().ends_with(".jpeg")
        && !key.to_lowercase().ends_with(".png")
        && !key.to_lowercase().ends_with(".gif")
        && !key.to_lowercase().ends_with(".tif")
        && !key.to_lowercase().ends_with(".tiff")
        && !key.to_lowercase().ends_with(".bmp")
    {
        return serve_s3(&state.s3_client, &key).await;
    }

    let target_key = format!("webp-cache/{}", key);

    match s3::exist_s3(&state.s3_client, &target_key).await {
        Ok(true) => return serve_s3(&state.s3_client, &target_key).await,
        Ok(false) => {}
        Err(err) => return err.into_response(),
    }

    match s3::exist_s3(&state.s3_client, &key).await {
        Ok(true) => return convert_and_serve(&state.s3_client, &key, &target_key, None).await,
        Ok(false) => {}
        Err(err) => return err.into_response(),
    }

    let full_path = format!("/{}", key);
    if let Some(thumb_req) = utils::parse_thumb_path(&full_path) {
        let source_key = format!(
            "wiki/{}{}/{}",
            thumb_req.archive_prefix.unwrap_or_default(),
            thumb_req.hash_path,
            thumb_req.original_filename
        );
        return convert_and_serve(
            &state.s3_client,
            &source_key,
            &target_key,
            Some(thumb_req.width),
        )
        .await;
    }

    (StatusCode::NOT_FOUND, "object not found".to_string()).into_response()
}

pub async fn handle_purge(
    State(state): State<Arc<AppState>>,
    Path(path_param): Path<String>,
) -> impl IntoResponse {
    let key = path_param;
    if key.starts_with("wiki/thumb/") {
        let full_path = format!("/{}", key);
        if utils::parse_thumb_path(&full_path).is_none() {
            return (StatusCode::BAD_REQUEST, "invalid request".to_string()).into_response();
        }
        return StatusCode::NO_CONTENT.into_response();
    }

    let target_key = format!("webp-cache/{}", key);

    match s3::exist_s3(&state.s3_client, &target_key).await {
        Ok(true) => {}
        Ok(false) => {
            return (StatusCode::NOT_FOUND, "webp cache not found".to_string()).into_response();
        }
        Err(err) => return err.into_response(),
    }

    if let Err(err) = s3::delete_s3(&state.s3_client, &target_key).await {
        return err.into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}

async fn handle_thumb_path(client: &Client, full_path: &str) -> Response {
    let thumb_req = match utils::parse_thumb_path(full_path) {
        Some(req) => req,
        None => return (StatusCode::BAD_REQUEST, "invalid request".to_string()).into_response(),
    };

    let archive_prefix = thumb_req.archive_prefix.as_deref().unwrap_or_default();
    let target_key = format!(
        "wiki/thumb/{}{}/{}/{}px-{}",
        archive_prefix,
        thumb_req.hash_path,
        thumb_req.original_filename,
        thumb_req.width,
        thumb_req.target_filename
    );

    match s3::exist_s3(client, &target_key).await {
        Ok(true) => return serve_s3(client, &target_key).await,
        Ok(false) => {}
        Err(err) => return err.into_response(),
    }

    let source_key = format!(
        "wiki/{}{}/{}",
        archive_prefix, thumb_req.hash_path, thumb_req.original_filename
    );

    match s3::read_from_s3(client, &source_key).await {
        Ok((data, _)) => match image_ops::process_image(data, Some(thumb_req.width)) {
            Ok((processed_data, content_type)) => {
                let upload_result =
                    s3::upload_to_s3(client, &target_key, processed_data.clone(), &content_type)
                        .await;
                image_response(upload_result, processed_data, content_type)
            }
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        },
        Err(err) => err.into_response(),
    }
}

async fn serve_s3(client: &Client, key: &str) -> Response {
    match s3::read_from_s3(client, key).await {
        Ok((data, content_type)) => {
            (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], data).into_response()
        }
        Err(err) => err.into_response(),
    }
}

async fn convert_and_serve(
    client: &Client,
    source_key: &str,
    target_key: &str,
    width: Option<u32>,
) -> Response {
    match s3::read_from_s3(client, source_key).await {
        Ok((data, _)) => match image_ops::process_image(data, width) {
            Ok((processed_data, content_type)) => {
                let upload_result =
                    s3::upload_to_s3(client, target_key, processed_data.clone(), &content_type)
                        .await;
                image_response(upload_result, processed_data, content_type)
            }
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        },
        Err(err) => err.into_response(),
    }
}

fn image_response(
    upload_result: Result<(), s3::S3HttpError>,
    processed_data: bytes::Bytes,
    content_type: String,
) -> Response {
    match upload_result {
        Ok(()) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type)],
            processed_data,
        )
            .into_response(),
        Err(err) => err.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn image_response_returns_uploaded_image() {
        let response = image_response(
            Ok(()),
            Bytes::from_static(b"webp"),
            "image/webp".to_string(),
        );

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "image/webp");
    }

    #[test]
    fn image_response_propagates_upload_error() {
        let response = image_response(
            Err(s3::S3HttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "cache unavailable",
            )),
            Bytes::from_static(b"webp"),
            "image/webp".to_string(),
        );

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
