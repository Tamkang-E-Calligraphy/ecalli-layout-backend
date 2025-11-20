use serde::Deserialize;

/// Request format for static layout
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutRequest {
    pub content: String,
    pub font_type: String,
    pub fixed_space: String,
    pub width: isize,
    pub height: isize,
    pub letter_space: isize,
    pub line_space: isize,
}

/// Response format for static layout
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutResponse {
    pub code: String,
    pub message: String,
    pub data: LayoutData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutData {
    pub letter_space: isize,
    pub line_space: isize,
    pub word: Vec<StaticSubject>,
}

/// Request format for downloading static letters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub subject: String,
    pub subject_font_type: String,
    pub subject_list: Vec<StaticSubject>,
    pub width: isize,
    pub height: isize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticSubject {
    pub pos_x: f64,
    pub pos_y: f64,
    pub width: isize,
    pub height: isize,
    pub line: isize,
}

/// Request format for creating poem animation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationRequest {
    pub subject: String,
    pub subject_font_type: String,
    pub subject_list: Vec<AnimateSubject>,
    pub content: String,
    pub font_type: String,
    pub word_list: Vec<AnimateSubject>,
    pub width: isize,
    pub height: isize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimateSubject {
    pub pos_x: f64,
    pub pos_y: f64,
    pub width: isize,
    pub height: isize,
}

