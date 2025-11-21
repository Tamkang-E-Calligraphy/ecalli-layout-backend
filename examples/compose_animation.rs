use ecalli_layout_backend::feature::json::{AnimateSubject, AnimationRequest};
use ecalli_layout_backend::feature::*;
use std::fs::File;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let test_req = AnimationRequest {
        subject: "".into(),
        subject_font_type: "楷書".into(),
        subject_list: Vec::new(),
        content: "安".into(),
        font_type: "草書".into(),
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
    let zip_buffer = zip_frames_to_memory(images)?;

    let output_filepath = "test_output.zip";
    let mut output_file = File::create(output_filepath)?;
    output_file.write_all(&zip_buffer)?;

    Ok(())
}
