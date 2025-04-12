#![feature(let_chains)]

use std::{path::PathBuf, sync::Arc};

use og_manager::OGManager;
use rocket::{fs::NamedFile, futures::StreamExt, get, post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use server_api::{
    db::{Database, Event},
    external::{
        toml,
        types::{
            api::{APIError, CompressedEvent},
            external::{chrono, serde_json},
            timing,
        },
        url::Url,
    },
    plugin::{PluginData, PluginTrait},
};

mod og;
mod og_manager;

pub struct Plugin {
    plugin_data: PluginData,
    og_manager: Arc<OGManager>,
    #[allow(unused)]
    config: ConfigData,
}

#[derive(Deserialize)]
struct ConfigData {
    pub save_location: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WebsiteVisit {
    client: String,
    website: Url,
}

impl PluginTrait for Plugin {
    async fn new(data: server_api::plugin::PluginData) -> Self
    where
        Self: Sized,
    {
        let config: ConfigData = toml::Value::try_into(
            data.config
                .clone()
                .expect("Failed to init web plugin! No config was provided!"),
        )
        .unwrap_or_else(|e| {
            panic!(
                "Unable to init web plugin! Provided config does not fit the requirements: {}",
                e
            )
        });

        Plugin {
            plugin_data: data,
            og_manager: Arc::new(OGManager::new(config.save_location.clone())),
            config,
        }
    }

    fn get_type() -> server_api::external::types::available_plugins::AvailablePlugins
    where
        Self: Sized,
    {
        server_api::external::types::available_plugins::AvailablePlugins::timeline_plugin_web
    }

    fn get_compressed_events(
        &self,
        query_range: &server_api::external::types::timing::TimeRange,
    ) -> std::pin::Pin<
        Box<
            dyn rocket::futures::Future<
                    Output = server_api::external::types::api::APIResult<
                        Vec<server_api::external::types::api::CompressedEvent>,
                    >,
                > + Send,
        >,
    > {
        let database = self.plugin_data.database.clone();
        let og_manager = self.og_manager.clone();
        let filter = Database::combine_documents(Database::generate_range_filter(query_range), Database::generate_find_plugin_filter(server_api::external::types::available_plugins::AvailablePlugins::timeline_plugin_web));
        Box::pin(async move {
            let mut cursor = database
                .get_events::<WebsiteVisit>()
                .find(filter, None)
                .await?;

            let mut result = Vec::new();
            while let Some(r) = cursor.next().await {
                let res = r?;
                let og_data = match og_manager.get_og(&res.event.website).await {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(APIError::Custom(format!(
                            "Unable to load open graph cache data: {}",
                            e
                        )));
                    }
                };
                result.push(CompressedEvent {
                    title: og_data.title.clone().unwrap_or(
                        res.event
                            .website
                            .host_str()
                            .unwrap_or("Website Visit")
                            .to_string(),
                    ),
                    time: res.timing,
                    data: serde_json::to_value((og_data, res.event)).unwrap(),
                });
            }

            Ok(result)
        })
    }

    fn rocket_build_access(
        &self,
        rocket: rocket::Rocket<rocket::Build>,
    ) -> rocket::Rocket<rocket::Build> {
        rocket.manage(self.og_manager.clone())
    }

    fn get_routes() -> Vec<rocket::Route>
    where
        Self: Sized,
    {
        routes![register_visit, app_icon]
    }
}

#[post("/register_visit", data = "<request>")]
async fn register_visit(
    database: &State<Arc<Database>>,
    request: Json<WebsiteVisit>,
    og_manager: &State<Arc<OGManager>>,
) -> Result<(), String> {
    if let Err(e) = og_manager.request_og(&request.website).await {
        println!("Request error: {}", e);
        return Err(format!(
            "Unable to save the open graph data to cache: {}",
            e
        ));
    }

    if let Err(e) = database.register_single_event(&Event {
        timing: timing::Timing::Instant(chrono::Utc::now()),
        id: format!("{}@{}", request.client, chrono::Utc::now()),
        plugin:
            server_api::external::types::available_plugins::AvailablePlugins::timeline_plugin_web,
        event: request.0,
    }).await {
        return Err(format!("Unable to register website visit to database: {}", e));
    }
    Ok(())
}

#[get("/image/<image_type>/<url>")]
pub async fn app_icon(
    image_type: &str,
    url: &str,
    og_manager: &State<Arc<OGManager>>,
) -> Option<NamedFile> {
    let url = match Url::parse(url) {
        Ok(v) => v,
        Err(_e) => return None,
    };
    let (_, _, _, favicon_path, image_path) = og_manager.get_paths(&url);
    match image_type {
        "og_image" => NamedFile::open(image_path).await.ok(),
        "favicon" => NamedFile::open(favicon_path).await.ok(),
        _ => None,
    }
}
