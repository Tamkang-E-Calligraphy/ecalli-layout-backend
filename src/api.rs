use crate::feature::json::AnimationRequest;
use crate::feature::*;
use actix_web::{HttpResponse, Responder, http::header, post, web};
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

    match compose_poem_animation_frames(body.into_inner()).await {
        Ok(images) => match zip_frames_to_memory(images) {
            Ok(zip_buffer) => {
                // Success: Provide a filename for ZIP archive.
                HttpResponse::Ok()
                    .content_type("application/zip")
                    .append_header((
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"result.zip\"",
                    ))
                    .body(zip_buffer)
            }
            Err(e) => HttpResponse::BadRequest().json(StatusResponse {
                code: "200".to_string(),
                message: format!("Internal error when compressing images into ZIP archive: {e}"),
            }),
        },
        Err(e) => HttpResponse::BadRequest().json(StatusResponse {
            code: "200".to_string(),
            message: format!("Internal error when generating poem animation: {e}"),
        }),
    }
}
