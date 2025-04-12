use server_api::external::tokio::fs::create_dir_all;
use server_api::external::tokio::fs::read_to_string;
use server_api::external::tokio::fs::try_exists;
use server_api::external::tokio::fs::write;
use server_api::external::tokio::fs::OpenOptions;
use server_api::external::tokio::io::AsyncWriteExt;
use server_api::external::types::external::reqwest;
use server_api::external::types::external::serde_json;
use std::io;
use std::path::PathBuf;

use server_api::external::url::Url;

use crate::og;
use crate::og::create_get_request;
use crate::og::OGData;
use crate::og::OGError;

pub struct OGManager {
    save_location: PathBuf,
}

impl OGManager {
    pub fn new(save_location: PathBuf) -> OGManager {
        OGManager { save_location }
    }

    pub async fn request_og(&self, url: &Url) -> Result<(), OGManagerError> {
        let (_, data_path, _, _, _) = self.get_paths(url);
        if !try_exists(data_path).await? {
            self.save_og(url, &og::get_og(url.clone()).await?).await?;
            self.save_favicon(url).await?;
            Ok(())
        } else {
            Ok(())
        }
    }

    pub async fn save_og(&self, url: &Url, og_data: &OGData) -> Result<(), OGManagerError> {
        let (path, data_path, _, _, image_path) = self.get_paths(url);
        create_dir_all(&path).await?;
        if let Some(image) = &og_data.image {
            let bytes = create_get_request(image.clone()).await?.bytes().await?;
            let mut image_path = path.clone();
            image_path.push(
                image
                    .path_segments()
                    .into_iter()
                    .last()
                    .unwrap()
                    .next()
                    .unwrap_or("image.png"),
            );
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .append(false)
                .open(image_path)
                .await?
                .write_all(&bytes)
                .await?;
        }

        write(data_path, &serde_json::to_string(og_data).unwrap()).await?;
        Ok(())
    }

    async fn save_favicon(&self, url: &Url) -> Result<(), OGManagerError> {
        let (_, _, _, favicon_path, _) = self.get_paths(url);
        if !try_exists(&favicon_path).await? {
            let mut favicon_url = url.clone();
            favicon_url.set_path("/favicon.ico");
            let bytes = create_get_request(favicon_url).await?.bytes().await?;
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .append(false)
                .open(favicon_path)
                .await?
                .write_all(&bytes)
                .await?;
        }
        Ok(())
    }

    pub async fn get_og(&self, url: &Url) -> Result<OGData, OGManagerError> {
        let (_path, data_path, _, _, _) = self.get_paths(url);
        Ok(serde_json::from_str(&read_to_string(data_path).await?)?)
    }

    pub fn get_paths(&self, url: &Url) -> (PathBuf, PathBuf, PathBuf, PathBuf, PathBuf) {
        let mut path = self.save_location.to_path_buf();
        path.push(url.host_str().unwrap_or("no_host"));
        let host_path = path.clone();
        let mut favicon_path = host_path.clone();
        favicon_path.push("favicon.ico");
        path.push(
            url.path()
                .chars()
                .map(|v| if v.is_alphanumeric() { v } else { '_' })
                .collect::<String>(),
        );
        path.push(
            url.query()
                .unwrap_or("")
                .chars()
                .map(|v| if v.is_alphanumeric() { v } else { '_' })
                .collect::<String>(),
        );
        let mut data_path = path.clone();
        data_path.push("data.json");
        let mut image_path = path.clone();
        image_path.push("image.png");
        (path, data_path, host_path, favicon_path, image_path)
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum OGManagerError {
    IOError(std::io::Error),
    RequestError(reqwest::Error),
    ParsingError(serde_json::Error),
    OGError(OGError),
}

impl std::error::Error for OGManagerError {}

impl std::fmt::Display for OGManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OGManagerError::IOError(e) => write!(f, "Unable to write or read og cache file: {}", e),
            OGManagerError::RequestError(e) => write!(f, "Unable to reqwest og image: {}", e),
            OGManagerError::ParsingError(e) => {
                write!(f, "Unable to parse open graph data from cache: {}", e)
            }
            OGManagerError::OGError(e) => write!(
                f,
                "Encounterd an og error while fetching the open graph data from website: {}",
                e
            ),
        }
    }
}

impl From<reqwest::Error> for OGManagerError {
    fn from(value: reqwest::Error) -> Self {
        OGManagerError::RequestError(value)
    }
}

impl From<io::Error> for OGManagerError {
    fn from(value: io::Error) -> Self {
        OGManagerError::IOError(value)
    }
}

impl From<serde_json::Error> for OGManagerError {
    fn from(value: serde_json::Error) -> Self {
        OGManagerError::ParsingError(value)
    }
}

impl From<OGError> for OGManagerError {
    fn from(value: OGError) -> Self {
        OGManagerError::OGError(value)
    }
}
