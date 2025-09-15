use ecalli_layout_backend::{AppError, CalliFont, BlobStorageSettings};
use std::io::{Cursor, Read};
use zip::ZipArchive;


#[tokio::main]
async fn main() -> Result<(), AppError> {
    let storage_config = BlobStorageSettings::from_local_env()?;
    let blob_client = storage_config.get_frame_client(CalliFont::Seal, '一')?;
    let binary = blob_client.download(None).await.unwrap();
    let cursor = Cursor::new(binary.into_raw_body().collect().await.unwrap());
    let zipfile = ZipArchive::new(cursor).unwrap();

    Ok(())

}
