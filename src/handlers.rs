use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    http::{StatusCode, header},
};
use aws_sdk_s3::Client;
use std::sync::Arc;
use percent_encoding::percent_decode_str;
use crate::{s3, image_ops, utils};

pub struct AppState {
    pub s3_client: Client,
}

pub async fn handle_resize(
    State(state): State<Arc<AppState>>,
    Path(path_param): Path<String>,
) -> impl IntoResponse {
    let path_param = percent_decode_str(&path_param).decode_utf8_lossy().to_string();
    let full_path = format!("/wiki/thumb/{}", path_param);
    
    let thumb_req = match utils::parse_thumb_path(&full_path) {
        Some(req) => req,
        None => return (StatusCode::BAD_REQUEST, "invalid request".to_string()).into_response(),
    };

    let target_key = format!(
        "wiki/thumb/{}{}/{}/{}px-{}",
        thumb_req.archive_prefix.clone().unwrap_or_default(),
        thumb_req.hash_path,
        thumb_req.original_filename,
        thumb_req.width,
        thumb_req.target_filename
    );

    // Check if target exists
    if s3::exist_s3(&state.s3_client, &target_key).await {
        return serve_s3(&state.s3_client, &target_key).await;
    }

    // Source key
    let source_key = format!(
        "wiki/{}{}/{}",
        thumb_req.archive_prefix.unwrap_or_default(),
        thumb_req.hash_path,
        thumb_req.original_filename
    );

    match s3::read_from_s3(&state.s3_client, &source_key).await {
        Ok((data, _)) => {
            match image_ops::process_image(data, Some(thumb_req.width)) {
                Ok((processed_data, content_type)) => {
                    // Upload to S3
                     if let Err(e) = s3::upload_to_s3(&state.s3_client, &target_key, processed_data.clone(), &content_type).await {
                         eprintln!("Failed to upload to S3: {}", e);
                     }
                    // Return result
                    (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, content_type)],
                        processed_data
                    ).into_response()
                },
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        },
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

pub async fn handle_webp(
    State(state): State<Arc<AppState>>,
    Path(path_param): Path<String>,
) -> impl IntoResponse {
    let key = percent_decode_str(&path_param).decode_utf8_lossy().to_string();
    
    // Check extension
    if !key.to_lowercase().ends_with(".jpg") && 
       !key.to_lowercase().ends_with(".jpeg") && 
       !key.to_lowercase().ends_with(".png") && 
       !key.to_lowercase().ends_with(".gif") &&
       !key.to_lowercase().ends_with(".tif") &&
       !key.to_lowercase().ends_with(".tiff") &&
       !key.to_lowercase().ends_with(".bmp") {
        return serve_s3(&state.s3_client, &key).await;
    }

    let target_key = format!("webp-cache/{}", key);

    if s3::exist_s3(&state.s3_client, &target_key).await {
        return serve_s3(&state.s3_client, &target_key).await;
    }

    // Check if source exists
    if s3::exist_s3(&state.s3_client, &key).await {
        // Convert to webp
        return convert_and_serve(&state.s3_client, &key, &target_key, None).await;
    }

    // Try parsing as thumb
    let full_path = format!("/{}", key);
    if let Some(thumb_req) = utils::parse_thumb_path(&full_path) {
        let source_key = format!(
            "wiki/{}{}/{}",
            thumb_req.archive_prefix.unwrap_or_default(),
            thumb_req.hash_path,
            thumb_req.original_filename
        );
        return convert_and_serve(&state.s3_client, &source_key, &target_key, Some(thumb_req.width)).await;
    }

    (StatusCode::NOT_FOUND, "object not found".to_string()).into_response()
}

pub async fn handle_purge(
    State(state): State<Arc<AppState>>,
    Path(path_param): Path<String>,
) -> impl IntoResponse {
    let key = percent_decode_str(&path_param).decode_utf8_lossy().to_string();
    let target_key = format!("webp-cache/{}", key);
    
    if !s3::exist_s3(&state.s3_client, &target_key).await {
        return (StatusCode::NOT_FOUND, "webp cache not found".to_string()).into_response();
    }
    
    if let Err(e) = s3::delete_s3(&state.s3_client, &target_key).await {
         return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    
    StatusCode::NO_CONTENT.into_response()
}

async fn serve_s3(client: &Client, key: &str) -> Response {
    match s3::read_from_s3(client, key).await {
        Ok((data, content_type)) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type)],
            data
        ).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(), // simplified
    }
}

async fn convert_and_serve(client: &Client, source_key: &str, target_key: &str, width: Option<u32>) -> Response {
    match s3::read_from_s3(client, source_key).await {
        Ok((data, _)) => {
            match image_ops::process_image(data, width) {
                 Ok((processed_data, content_type)) => {
                     // Upload
                     if let Err(e) = s3::upload_to_s3(client, target_key, processed_data.clone(), &content_type).await {
                         eprintln!("Failed to upload webp to S3: {}", e);
                     }
                     (
                        StatusCode::OK,
                        [(header::CONTENT_TYPE, content_type)],
                        processed_data
                    ).into_response()
                 },
                 Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        },
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}
