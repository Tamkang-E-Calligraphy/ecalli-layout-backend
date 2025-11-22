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
        // Set to 30 FPS with 33ms delay when encoding webp image.
        Ok(images) => match encode_frames_to_webp(images, 33) {
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
                message: format!("Internal error when encoding frames to WebP format: {e}"),
            }),
        },
        Err(e) => HttpResponse::BadRequest().json(StatusResponse {
            code: "200".to_string(),
            message: format!("Internal error when generating stroke animation: {e}"),
        }),
    }
}
