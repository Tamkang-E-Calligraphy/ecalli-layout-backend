use crate::{
    DB, KEY,
    feature::{
        json::{AnimationRequest, CheckStatus},
        *,
    },
};
use actix_web::{HttpResponse, Responder, http::header, post, web};
use fjall::KeyspaceCreateOptions;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub code: String,
    pub message: String,
}

#[post("/generate-animation")]
pub async fn handle_poem_animation_generation(body: web::Json<AnimationRequest>) -> impl Responder {
    // Restrict the canvas size to below 4096x4096.
    if body.width > 4096 || body.height > 4096 {
        return HttpResponse::BadRequest().json(StatusResponse {
            code: "200".to_string(),
            message: "Canvas dimensions too large.".to_string(),
        });
    }

    // Open a tree with default keyspace.
    match DB
        .get()
        .unwrap()
        .keyspace(KEY, KeyspaceCreateOptions::default)
    {
        Ok(tree) => match generate_poem_animation_webp(body.into_inner(), &tree).await {
            Ok(webp_data) => {
                // Success: Provide a filename for WebP Image.
                HttpResponse::Ok()
                    .content_type("image/webp")
                    .append_header((
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"result.webp\"",
                    ))
                    .body(webp_data.to_vec())
            }
            Err(e) => HttpResponse::BadRequest().json(StatusResponse {
                code: "200".to_string(),
                message: format!("Internal error: {e}"),
            }),
        },

        Err(e) => HttpResponse::BadRequest().json(StatusResponse {
            code: "200".to_string(),
            message: format!("Internal error: {e}"),
        }),
    }
}

#[post("/progress/update")]
pub async fn get_download_progress(body: web::Json<CheckStatus>) -> impl Responder {
    match DB
        .get()
        .unwrap()
        .keyspace(KEY, KeyspaceCreateOptions::default)
    {
        Ok(tree) => match tree.get(body.task_id.as_str()) {
            Ok(Some(progress_bytes)) => {
                // Success: Parse progress to JSON integer for progress field.
                HttpResponse::Ok()
                    .json(
                        CheckStatus {
                            task_id: body.task_id.clone(),
                            progress: isize::from_be_bytes(*progress_bytes.as_array().unwrap()),
                        }
                    )
            },
            Ok(None) => HttpResponse::BadRequest().json(StatusResponse {
                code: "200".to_string(),
                message: "Internal error: Cannot fetch value by the given Task ID, please make sure the task is valid.".to_string(),
            }),
            Err(e) => HttpResponse::BadRequest().json(StatusResponse {
                code: "200".to_string(),
                message: format!("Internal error: {e}"),
            }),
        },
            Err(e) => HttpResponse::BadRequest().json(StatusResponse {
                code: "200".to_string(),
                message: format!("Internal error: {e}"),
            }),

    }
}
