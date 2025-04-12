use std::{collections::HashMap, fmt::Display};

use html_parser::Node;
use htmlentity::entity::ICodedDataTrait;
use serde::{Deserialize, Serialize};
use server_api::external::{
    types::external::reqwest::{self, Response},
    url::Url,
};

#[derive(Serialize, Deserialize)]
pub struct OGData {
    pub title: Option<String>,
    pub image: Option<Url>,
    pub url: Url,
    pub description: Option<String>,
    pub site_name: Option<String>,
}

pub async fn get_og(url: Url) -> Result<OGData, OGError> {
    let request = create_get_request(url.clone()).await?;
    Ok(extract_og(request.text().await?, url)?)
}

pub fn extract_og(html: String, source_url: Url) -> Result<OGData, html_parser::Error> {
    let dom = html_parser::Dom::parse(&html)?;
    let mut result = HashMap::new();
    deep_search_meta(&dom.children, &mut result);
    Ok(OGData {
        title: result.get("title").map(|v| v.to_string()),
        image: result
            .get("image")
            .and_then(|v| Url::parse(v).or(source_url.clone().join(v)).ok()),
        description: result.get("description").map(|v| v.to_string()),
        site_name: result.get("site_name").map(|v| v.to_string()),
        url: result
            .get("url")
            .and_then(|v| Url::parse(v).or(source_url.clone().join(v)).ok())
            .unwrap_or(source_url),
    })
}

fn deep_search_meta<'a>(nodes: &'a [Node], result: &mut HashMap<&'a str, String>) {
    nodes.iter().for_each(|node| {
        if let Node::Element(e) = node {
            if e.name == "meta"
                && let Some(Some(property)) = e.attributes.get("property")
                && let Some(Some(content)) = e.attributes.get("content")
            {
                if let Some(og_property) = property.strip_prefix("og:") {
                    result.insert(
                        og_property,
                        htmlentity::entity::decode(content.as_bytes())
                            .to_string()
                            .unwrap_or(content.clone()),
                    );
                }
            } else {
                deep_search_meta(&e.children, result);
            }
        }
    });
}

#[derive(Debug)]
pub enum OGError {
    RequestError(reqwest::Error),
    HTMLError(html_parser::Error),
}

impl From<reqwest::Error> for OGError {
    fn from(value: reqwest::Error) -> Self {
        OGError::RequestError(value)
    }
}

impl From<html_parser::Error> for OGError {
    fn from(value: html_parser::Error) -> Self {
        OGError::HTMLError(value)
    }
}

impl std::error::Error for OGError {}

impl Display for OGError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OGError::HTMLError(h) => write!(f, "Unable to parse the html: {}", h),
            OGError::RequestError(r) => write!(f, "Unable to request the specified url: {}", r),
        }
    }
}

pub async fn create_get_request(
    url: Url,
) -> Result<Response, server_api::external::types::external::reqwest::Error> {
    reqwest::Client::new().get(url.clone())
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .send().await
}
