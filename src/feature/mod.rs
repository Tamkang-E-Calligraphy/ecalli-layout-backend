pub mod json;
use json::*;

use std::fmt;
use std::io::{Cursor, Read};
use std::path::Path;
use std::str::FromStr;

use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use image::{
    ExtendedColorType, ImageEncoder, Rgba, RgbaImage,
    codecs::png::PngEncoder,
    imageops::{self, FilterType},
};
use webp_animation::{Encoder, WebPData};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    DotEnvFailure(#[from] dotenv::Error),
    #[error("Azure SDK error: {0}")]
    AzureSdkFailure(String),
    #[error("Azure stoage error: {0}")]
    AzureStorageFailure(#[from] azure_storage::Error),
    #[error(transparent)]
    ZipFailure(#[from] zip::result::ZipError),
    #[error(transparent)]
    WebpFailure(#[from] webp_animation::Error),
    #[error("Invalid file name extracted from zip archive: {0}")]
    InvalidFileName(String),
    #[error(transparent)]
    ImageOpsFailure(#[from] image::ImageError),
    #[error("Cannot encode a list of empty frames.")]
    EmptyFrame,
    #[error("I/O Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid subject font type: {0}")]
    InvalidFontType(String),
}

pub enum CalliFont {
    Clerical,
    Cursive,
    Regular,
    Seal,
    SemiCursive,
}

impl FromStr for CalliFont {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "楷書" => Ok(CalliFont::Regular),
            "草書" => Ok(CalliFont::Cursive),
            "行書" => Ok(CalliFont::SemiCursive),
            "隸書" => Ok(CalliFont::Clerical),
            "篆書" => Ok(CalliFont::Seal),
            _ => Err(AppError::InvalidFontType(s.to_string())),
        }
    }
}

impl fmt::Display for CalliFont {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CalliFont::Clerical => write!(f, "Clerical"),
            CalliFont::Cursive => write!(f, "Cursive"),
            CalliFont::Regular => write!(f, "Regular"),
            CalliFont::Seal => write!(f, "Seal"),
            CalliFont::SemiCursive => write!(f, "SemiCursive"),
        }
    }
}

pub struct BlobStorageConfig {
    pub account: String,
    pub access_key: String,
    pub container: String,
}

impl BlobStorageConfig {
    pub fn from_local_env() -> Result<Self, AppError> {
        Ok(BlobStorageConfig {
            account: dotenv::var("STORAGE_ACCOUNT")?,
            access_key: dotenv::var("STORAGE_ACCESS_KEY")?,
            container: dotenv::var("STORAGE_CONTAINER")?,
        })
    }

    pub fn set_container_name(&mut self, name: &str) {
        self.container = name.to_string();
    }

    pub fn get_static_font_client(&self, font_type: CalliFont, font_name: char) -> BlobClient {
        let blob_name = format!("{font_type}/{font_name}.png");
        let storage_credit =
            StorageCredentials::access_key(self.account.clone(), self.access_key.clone());
        let service_client = BlobServiceClient::new(&self.account, storage_credit);

        service_client
            .container_client(self.container.clone())
            .blob_client(blob_name)
    }

    pub fn get_frame_client(&self, font_type: &CalliFont, zip_name: char) -> BlobClient {
        //let primary_endpoint = format!("https://{}.blob.core.windows.net/", self.account);
        let blob_name = format!("{font_type}/{zip_name}.zip");
        let storage_credit =
            StorageCredentials::access_key(self.account.clone(), self.access_key.clone());
        let service_client = BlobServiceClient::new(&self.account, storage_credit);

        service_client
            .container_client(self.container.clone())
            .blob_client(blob_name)
    }

    async fn get_poem_frames_by_font_type(
        &self,
        font_type: &CalliFont,
        content: &str,
    ) -> Result<Vec<Vec<WordFrame>>, AppError> {
        let mut result = Vec::with_capacity(content.len());
        for word in content.chars() {
            // Ensure the frontend has already excluded none Chinese letters.
            if matches!(word, '，' | '。' | '？' | '！' | ',' | '?' | '!') {
                unreachable!();
            }

            // Download request to BLOB storage.
            let blob_client = self.get_frame_client(font_type, word);
            // Check if the selected word exists.
            if blob_client.exists().await? {
                let word_frames = WordFrame::load_from_client(blob_client).await?;
                result.push(word_frames);
            } else {
                // Todo: Replace with an actual static frame of the whole word.
                result.push(vec![WordFrame {
                    name: word,
                    img: RgbaImage::new(0, 0),
                    width: 0,
                    height: 0,
                    pos_x: 0,
                    pos_y: 0,
                }]);
            }
        }

        Ok(result)
    }
}

// Greedily distributes words into K columns sequentially (in input order)
// to minimize the max height (H_max).
//
// Returns: (H_max, layout_map)
fn calculate_layout_data(heights: &[f64], k: usize) -> (f64, Vec<usize>) {
    let mut current_column_heights = vec![0.0; k];
    let mut layout_map = Vec::with_capacity(heights.len());

    for &h in heights {
        // Find the index of the shortest column
        let mut min_height = f64::INFINITY;
        let mut min_index = 0;

        for (i, &col_height) in current_column_heights.iter().enumerate() {
            if col_height < min_height {
                min_height = col_height;
                min_index = i;
            }
        }

        current_column_heights[min_index] += h;
        layout_map.push(min_index);
    }

    let h_max = current_column_heights.into_iter().fold(0.0, f64::max);
    (h_max, layout_map)
}

// Calculates the final scaled (x, y) coordinate for the top-left corner of each word,
// returning them as u32 pixel values after rounding.
fn get_word_coordinates(
    word_width: u32,
    word_heights: &[u32],
    s: f64,
    k: usize,
    final_layout_map: &[usize],
) -> Vec<(u32, u32)> {
    let word_width_f64 = word_width as f64;
    let mut column_offsets = vec![0.0; k]; // Keep f64 offsets for precision
    let mut scaled_coords = Vec::with_capacity(word_heights.len());
    let scaled_word_width = word_width_f64 * s;

    for (i, &h) in word_heights.iter().enumerate() {
        let h_f64 = h as f64;
        let column_index = final_layout_map[i];

        // X coordinate (f64 calculation)
        let x_f64 = column_index as f64 * scaled_word_width;

        // Y coordinate (f64 calculation)
        let y_f64 = column_offsets[column_index];

        // Convert to u32 pixel coordinates
        scaled_coords.push((x_f64 as u32, y_f64 as u32));

        // Update the column offset
        let scaled_height = h_f64 * s;
        column_offsets[column_index] += scaled_height;
    }

    scaled_coords
}

/*
pub async fn compose_poem_static_layout(mut selected_words: Vec<WordFrame>, canvas_width: u32, canvas_height: u32, fixed_space: bool) -> Result<(), AppError> {
    let word_orig_width = selected_words[0].width;
    let word_count = selected_words.len();
    if fixed_space {
        let word_orig_height = selected_words.iter().max_by(|x, y| x.height.cmp(&y.height));


    } else {
        // Convert u32 inputs to f64 for calculations involving ratios and scaling
        let canvas_width_f64 = canvas_width as f64;
        let canvas_height_f64 = canvas_height as f64;
        let word_width_f64 = word_orig_width as f64;
        // Convert heights for the helper function
        let word_heights_f64: Vec<f64> = selected_words.iter().map(|&w| w.height as f64).collect();

        let mut max_scaling_factor_s = 0.0;
        let mut optimal_columns_k = 1;
        let mut optimal_h_max = f64::INFINITY;

        // Iterate through all possible column counts (K) from 1 up to N
        (1..=word_count).for_each(|k| {

        });
    }

    todo!();
}
*/

pub async fn generate_poem_animation_webp(
    req: AnimationRequest,
    frame_delay_ms: i32,
) -> Result<WebPData, AppError> {
    let canvas_width = req.width as u32;
    let canvas_height = req.height as u32;
    let font_type = CalliFont::from_str(&req.font_type)?;
    let blob_config = BlobStorageConfig::from_local_env()?;

    let mut content_strokes = blob_config
        .get_poem_frames_by_font_type(&font_type, &req.content)
        .await?;

    let mut main_canvas =
        RgbaImage::from_pixel(canvas_width, canvas_height, Rgba([255, 255, 255, 255]));

    // Initialize the WebP Encoder with default config.
    let mut encoder = Encoder::new((canvas_width, canvas_height))?;
    let mut current_timestamp = 0;

    if req.word_list.len() == content_strokes.len() {
        for (strokes, layer) in content_strokes.iter_mut().zip(req.word_list.iter()) {
            // Process word with valid frames only.
            if strokes.len() > 1 {
                for frame in strokes {
                    frame.resize_img_by_size(layer.width, layer.height);
                    // Apply strokes onto the main canvas
                    imageops::overlay(
                        &mut main_canvas,
                        &frame.img,
                        layer.pos_x as i64,
                        layer.pos_y as i64,
                    );
                    // Add the word frame to encoder.
                    encoder.add_frame(main_canvas.as_raw(), current_timestamp)?;
                    // Advance the timestamp by the frame delay for the next frame
                    current_timestamp += frame_delay_ms;
                }
            }
        }
    } else {
        return Err(AppError::InvalidFileName(
            "WordList contains illegal characters".to_string(),
        ));
    }

    // Finalize the animation
    // The last timestamp tells the encoder the total duration.
    let webp_bytes = encoder.finalize(current_timestamp)?;

    Ok(webp_bytes)
}

pub async fn compose_poem_animation_frames(
    req: AnimationRequest,
) -> Result<Vec<RgbaImage>, AppError> {
    let canvas_width = req.width as u32;
    let canvas_height = req.height as u32;
    let mut recorded_frames = Vec::new();
    let font_type = CalliFont::from_str(&req.font_type)?;
    let blob_config = BlobStorageConfig::from_local_env()?;

    let mut content_strokes = blob_config
        .get_poem_frames_by_font_type(&font_type, &req.content)
        .await?;

    let mut main_canvas =
        RgbaImage::from_pixel(canvas_width, canvas_height, Rgba([255, 255, 255, 255]));

    if req.word_list.len() == content_strokes.len() {
        for (strokes, layer) in content_strokes.iter_mut().zip(req.word_list.iter()) {
            // Process word with valid frames only.
            if strokes.len() > 1 {
                for frame in strokes {
                    frame.resize_img_by_size(layer.width, layer.height);
                    imageops::overlay(
                        &mut main_canvas,
                        &frame.img,
                        layer.pos_x as i64,
                        layer.pos_y as i64,
                    );
                    // Collect the canvas frame.
                    recorded_frames.push(main_canvas.clone());
                }
            }
        }
    } else {
        return Err(AppError::InvalidFileName(
            "WordList contains illegal characters".to_string(),
        ));
    }

    Ok(recorded_frames)
}

/// Converts a Vec of Rgba<u8> frames into an animated WebP byte array.
/// The `frame_delay_ms` is exchangable to FPS based on the formula:
/// 1000 / FPS = frame_delay_ms (e.g. 33.3ms delay is roughly 30 FPS)
pub fn encode_frames_to_webp(
    frames: Vec<RgbaImage>,
    frame_delay_ms: i32, // Time each frame is displayed
) -> Result<WebPData, AppError> {
    // Check if there are frames to encode
    let first_frame = match frames.first() {
        Some(f) => f,
        None => return Err(AppError::EmptyFrame),
    };

    // Grab the canvas width and height from the first frame.
    let (width, height) = first_frame.dimensions();

    // Initialize the WebP Encoder with default config.
    // Default ColorMode = RgbA
    let mut encoder = Encoder::new((width, height))?;

    let mut current_timestamp = 0; // The timestamp tracker for frame duration

    for frame in frames {
        // Add each frame to the encoder
        // The timestamp tells the encoder WHEN the frame is presented.
        encoder.add_frame(
            frame.as_raw(), // Raw pixel data buffer, default to RgbA
            current_timestamp,
        )?;

        // Advance the timestamp by the frame delay for the next frame
        current_timestamp += frame_delay_ms;
    }

    // Finalize the animation
    // The last timestamp tells the encoder the total duration.
    let webp_bytes = encoder.finalize(current_timestamp)?;

    Ok(webp_bytes)
}

/// Example canvas frame filename: frame_001.png
pub fn zip_frames_to_memory(frames: Vec<RgbaImage>) -> Result<Vec<u8>, AppError> {
    // Create a buffer to hold the final ZIP file in memory
    let mut zip_buffer = Vec::new();

    // Use a scope to ensure the borrow of zip_buffer ends when we need to return it
    {
        let mut zip_writer = ZipWriter::new(Cursor::new(&mut zip_buffer));

        // OPTIMIZATION: Use "Stored" (No Compression) for the ZIP container.
        // Why? PNGs are already compressed. Compressing them again (Deflate)
        // wastes CPU for almost zero size gain.
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Stored)
            .unix_permissions(0o755);

        for (index, frame) in frames.iter().enumerate() {
            let filename = format!("frame_{index:03}.png"); // e.g., frame_001.png

            // Start a file entry in the ZIP
            zip_writer.start_file(&filename, options)?;

            // Encode the pixels to PNG and write directly into the Zip stream
            // This is the memory-saving trick. We don't create a Vec<u8> for the PNG first.
            let (width, height) = frame.dimensions();
            let encoder = PngEncoder::new(&mut zip_writer);

            encoder.write_image(
                frame.as_raw(), // The raw pixel bytes
                width,
                height,
                ExtendedColorType::Rgba8, // IMPORTANT: Tell PNG this is 8-bit RgbA
            )?;
        }

        // Finish the ZIP structure
        zip_writer.finish()?;
    }

    Ok(zip_buffer)
}

pub struct WordFrame {
    pub name: char,
    pub img: RgbaImage,
    pub width: u32,
    pub height: u32,
    pub pos_x: u64,
    pub pos_y: u64,
}

impl WordFrame {
    // Load the static drawing of the word provided by the blob client.
    pub async fn load_static_from_client(client: BlobClient) -> Result<Self, AppError> {
        let fpath = Path::new(client.blob_name());
        let blob = client
            .get_content()
            .await
            .map_err(|e| AppError::AzureSdkFailure(e.to_string()))?;
        if fpath.extension().is_some_and(|ext| {
            let ext_str = ext.to_ascii_lowercase();
            ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png"
        }) && let Some(char_name) = fpath
            .file_stem()
            .and_then(|oss| oss.to_str().and_then(|s| s.parse::<char>().ok()))
        {
            // Use load_from_memory to infer format (JPG or PNG)
            let rgba_img = image::load_from_memory(&blob)?;
            let height = rgba_img.height();
            let width = rgba_img.width();

            Ok(Self {
                name: char_name,
                img: rgba_img.into(),
                width,
                height,
                pos_x: 0,
                pos_y: 0,
            })
        } else {
            Err(AppError::InvalidFileName(
                fpath.to_str().unwrap().to_string(),
            ))
        }
    }

    /// Loads all numbered JPG/PNG frames from a specific word's zip archive provided as a byte blob.
    /// Assumes filenames inside the zip are in the format "FrameNumber.jpg" (e.g., "1.jpg", "10.jpg").
    /// Returns a sorted vector of `Self` containing `RgbaImage` buffers and other metadata.
    pub async fn load_from_client(client: BlobClient) -> Result<Vec<Self>, AppError> {
        let name_parts: Vec<&str> = Path::new(client.blob_name())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .collect();
        let char_name = name_parts[0].parse::<char>().unwrap();
        let blob = client
            .get_content()
            .await
            .map_err(|e| AppError::AzureSdkFailure(e.to_string()))?;
        let mut zipfile = ZipArchive::new(Cursor::new(blob))?;

        let frames_with_ids_res = (0..zipfile.len())
            .map(|idx| {
                let mut file = zipfile.by_index(idx)?;
                let fname = file.name();
                let fpath = Path::new(fname);
                if fpath.extension().is_some_and(|ext| {
                    let ext_str = ext.to_ascii_lowercase();
                    ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png"
                }) && let Some(frame_id) = fpath
                    .file_stem()
                    .and_then(|oss| oss.to_str().and_then(|s| s.parse::<u32>().ok()))
                {
                    let mut imgbuf = Vec::new();
                    file.read_to_end(&mut imgbuf)?;

                    // Use load_from_memory to infer format (JPG or PNG)
                    let rgba_img = image::load_from_memory(&imgbuf)?;
                    let height = rgba_img.height();
                    let width = rgba_img.width();

                    Ok((
                        frame_id,
                        Self {
                            name: char_name,
                            img: rgba_img.into(),
                            height,
                            width,
                            pos_x: 0,
                            pos_y: 0,
                        },
                    ))
                } else {
                    Err(AppError::InvalidFileName(fname.to_string()))
                }
            })
            .collect::<Result<Vec<(u32, Self)>, AppError>>();
        // Sort the frames by file name
        let mut frames_with_ids = frames_with_ids_res?;
        frames_with_ids.sort_by_key(|(id, _)| *id);

        Ok(frames_with_ids
            .into_iter()
            .map(|(_, frame)| frame)
            .collect())
    }

    pub fn resize_img_by_scale(&mut self, scale: f64) {
        let new_w = (self.width as f64 * scale) as u32;
        let new_h = (self.height as f64 * scale) as u32;
        let new_img = imageops::resize(&self.img, new_w, new_h, FilterType::Gaussian);

        self.width = new_w;
        self.height = new_h;
        self.img = new_img;
    }

    pub fn resize_img_by_size(&mut self, resize_width: isize, resize_height: isize) {
        let new_w = resize_width as u32;
        let new_h = resize_height as u32;
        let new_img = imageops::resize(&self.img, new_w, new_h, FilterType::Gaussian);

        self.width = new_w;
        self.height = new_h;
        self.img = new_img;
    }

    // Return `true` if the word frame is empty.
    pub fn is_empty(&self) -> bool {
        self.width == self.height && self.width == 0
    }
}
