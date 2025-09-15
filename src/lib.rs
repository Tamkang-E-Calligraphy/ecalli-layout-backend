use azure_storage_blob::{BlobClient, BlobClientOptions};
use azure_identity::DefaultAzureCredential;
use secrecy::SecretString;
use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    DotEnvFailure(#[from] dotenv::Error),
    #[error("Azure SDK error")]
    AzureSdkFailure,
}

pub enum CalliFont {
    Clerical,
    Cursive,
    Regular,
    Seal,
    SemiCurve,
}

impl fmt::Display for CalliFont {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            CalliFont::Clerical => write!(f, "Clerical"),
            CalliFont::Cursive => write!(f, "Cursive"),
            CalliFont::Regular => write!(f, "Regular"),
            CalliFont::Seal => write!(f, "Seal"),
            CalliFont::SemiCurve => write!(f, "SemiCurve"),
        }
    }
}

pub struct BlobStorageSettings {
    pub account: String,
    pub access_key: SecretString,
    pub container: String,
}

impl BlobStorageSettings {
    pub fn from_local_env() -> Result<Self, AppError> {
        Ok(BlobStorageSettings {
            account: dotenv::var("STORAGE_ACCOUNT")?,
            access_key: SecretString::from(dotenv::var("STORAGE_ACCESS_KEY")?),
            container: dotenv::var("STORAGE_CONTAINER")?,
        })
    }

    pub fn set_container_name(&mut self, name: &str) {
        self.container = name.to_string();
    }

    pub fn get_frame_client(&self, font_type: CalliFont, zip_name: char) -> Result<BlobClient, AppError> {
        let primary_endpoint = format!("https://{}.blob.core.windows.net/", self.account);
        let blob_name = format!("{font_type}/{zip_name}.zip");

        let client = BlobClient::new(
            &primary_endpoint,
            self.container.clone(),
            blob_name,
            DefaultAzureCredential::new().map_err(|e| { eprintln!("{e}"); AppError::AzureSdkFailure})?,
            Some(BlobClientOptions::default()),
        ).map_err(|e| { eprintln!("{e}"); AppError::AzureSdkFailure })?;

        Ok(client)
    }
}

