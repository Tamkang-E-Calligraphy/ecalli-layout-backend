use ecalli_layout_backend::feature::json::{AnimateSubject, AnimationRequest};
use ecalli_layout_backend::feature::*;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let test_req = AnimationRequest {
        subject: "".into(),
        subject_font_type: "楷書".into(),
        subject_list: Vec::new(),
        content: "安".into(),
        font_type: "行書".into(),
        word_list: vec![AnimateSubject {
            pos_x: 0.,
            pos_y: 0.,
            width: 720,
            height: 720,
        }],
        width: 800,
        height: 800,
    };
    let images = compose_poem_animation_frames(test_req).await?;
    // Compress all frame PNG images into a zip archive.
    let zip_buffer = zip_frames_to_memory(images)?;
    fs::write("test_output.zip", zip_buffer)?;
    // Compose all frame rgba pixels into a webp image.
    let webpdata = encode_frames_to_webp(images, 33)?;
    fs::write("test_output.webp", webpdata)?;

    Ok(())
}
