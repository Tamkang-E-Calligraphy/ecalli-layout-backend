use azure_storage::prelude::*;
use azure_storage_blobs::prelude::*;
use image::{DynamicImage, Rgba, RgbaImage, imageops};
use std::fmt;
use std::io::{Cursor, Read};
use std::str::FromStr;
use std::path::Path;
use zip::ZipArchive;
use serde::Deserialize;

/// Request format for static layout
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticLayoutRequest {
    pub subject: String,
    pub subject_font_type: String,
    pub subject_list: Vec<StaticSubject>,
    pub width: usize,
    pub height: usize,

}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticSubject {
    pub pos_x: f64,
    pub pos_y: f64,
    pub width: usize,
    pub height: usize,
}


#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    DotEnvFailure(#[from] dotenv::Error),
    #[error("Azure SDK error: {0}")]
    AzureSdkFailure(String),
    #[error(transparent)]
    ZipFailure(#[from] zip::result::ZipError),
    #[error("Invalid file name extracted from zip archive: {0}")]
    InvalidFileName(String),
    #[error(transparent)]
    ImageOpsFailure(#[from] image::ImageError),
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

    pub fn get_frame_client(&self, font_type: CalliFont, zip_name: char) -> BlobClient {
        //let primary_endpoint = format!("https://{}.blob.core.windows.net/", self.account);
        let blob_name = format!("{font_type}/{zip_name}.zip");
        let storage_credit =
            StorageCredentials::access_key(self.account.clone(), self.access_key.clone());
        let service_client = BlobServiceClient::new(&self.account, storage_credit);

        service_client
            .container_client(self.container.clone())
            .blob_client(blob_name)
    }

    /*
    async fn get_poem_frames_by_font_type(&self, font_type: CalliFont, poem: Vec<char>) -> Result<Vec<Vec<WordFrame>>, AppError> {
        let mut result = Vec::with_capacity(poem.len());
        for word in poem {
            if matches!(word, '，' | '。' | '？' | '！' | ',' | '?' | '!') { continue; }


            match self.get_frame_client(font_type, word).get_content() {
                Ok(blob) => {

                },
                Err(e) => {
                    eprintln!("{e}");

                }
            }
        }

    }
    */
}

pub fn compose_poem_animation_frames(
    mut selected_frames: Vec<Vec<WordFrame>>,
    canvas_width: u32,
    canvas_height: u32,
) -> Result<RgbaImage, AppError> {
    let blank_canvs =
        RgbaImage::from_pixel(canvas_width, canvas_height, Rgba([255, 255, 255, 255]));
    todo!();
}

pub struct WordFrame {
    pub name: char,
    pub img: DynamicImage,
    pub height: u32,
    pub width: u32,
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
            let img = image::load_from_memory(&blob)?;
            let height = img.height();
            let width = img.width();

            Ok(Self {
                name: char_name,
                img,
                height,
                width,
                pos_x: 0,
                pos_y: 0,
            })
        } else {
            Err(AppError::InvalidFileName(
                fpath.to_str().unwrap().to_string(),
            ))
        }
    }

    // Loads all numbered JPG/PNG frames from a specific word's zip archive provided as a byte blob.
    // Assumes filenames inside the zip are in the format "FrameNumber.jpg" (e.g., "1.jpg", "10.jpg").
    // Returns a sorted vector of `DynamicImage` frames for that word. along with the dimensions
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

        (0..zipfile.len())
            .map(|idx| {
                let mut file = zipfile.by_index(idx)?;
                let fname = file.name();
                let fpath = Path::new(fname);
                if fpath.extension().is_some_and(|ext| {
                    let ext_str = ext.to_ascii_lowercase();
                    ext_str == "jpg" || ext_str == "jpeg" || ext_str == "png"
                }) && fpath
                    .file_stem()
                    .is_some_and(|oss| oss.to_str().is_some_and(|s| s.parse::<u32>().is_ok()))
                {
                    let mut imgbuf = Vec::new();
                    file.read_to_end(&mut imgbuf)?;

                    // Use load_from_memory to infer format (JPG or PNG)
                    let img = image::load_from_memory(&imgbuf)?;
                    let height = img.height();
                    let width = img.width();

                    Ok(Self {
                        name: char_name,
                        img,
                        height,
                        width,
                        pos_x: 0,
                        pos_y: 0,
                    })
                } else {
                    Err(AppError::InvalidFileName(fname.to_string()))
                }
            })
            .collect()
    }
}
