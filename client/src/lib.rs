use {
    client_api::{
        api::{encode_url_component, relative_url},
        external::url::Url,
        plugin::{PluginData, PluginEventData, PluginTrait},
        result::EventResult,
        style::Style,
    },
    leptos::{view, IntoView, View},
    serde::Deserialize,
};

pub struct Plugin {}

impl PluginTrait for Plugin {
    fn get_style(&self) -> Style {
        Style::Acc1
    }

    async fn new(_data: PluginData) -> Self
    where
        Self: Sized,
    {
        Plugin {}
    }

    fn get_component(
        &self,
        data: PluginEventData,
    ) -> EventResult<Box<dyn FnOnce() -> leptos::View>> {
        let (mut data, visit) = data.get_data::<(OGData, WebsiteVisit)>()?;
        data.image = data.image.map(|_v| {
            relative_url(&format!(
                "/api/plugin/timeline_plugin_web/image/og_image/{}",
                encode_url_component(visit.website.as_str())
            ))
            .unwrap()
        });
        Ok(Box::new(move || -> View {
            view! {
                    <div style="box-sizing: border-box; display: flex; gap: calc(var(--contentSpacing) * 0.5); flex-direction: column; width: 100%; padding: calc(var(--contentSpacing) * 0.5); background-color: var(--accentColor1);align-items: start;color: var(--lightColor)">
                        <div style="display: flex; align-items: center; flex-direction: row; gap: calc(var(--contentSpacing) * 0.3)"><img style="height: calc(var(--contentSpacing) * 2);" src=format!("/api/plugin/timeline_plugin_web/image/favicon/{}", encode_url_component(visit.website.clone().as_str()))/><b>{data.site_name.clone().unwrap_or(data.url.clone().host_str().unwrap_or("").to_string())}</b></div>
                        {
                            match data.title {
                                Some(title) => {
                                    view! {
                                        <h3>{title}</h3>
                                    }.into_view()
                                }
                                None => {
                                    view! {}.into_view()
                                }
                            }
                        }
                        {
                            match data.description {
                                Some(desc) => {
                                    view!{
                                        <a style="font-weight: lighter; font-size: small;">{desc}</a>
                                    }.into_view()
                                }
                                None => {
                                    view! {}.into_view()
                                }
                            }
                        }
                        {
                            match data.image {
                                Some(img) => {
                                    view! {
                                        <img style="max-width: 100%" src=img.to_string() />
                                    }.into_view()
                                }
                                None => {
                                    view! {}.into_view()
                                }
                            }
                        }
                        <a style="font-weight: lighter; color: var(--lightColor); font-size: small;" href=visit.website.to_string()>{format!("{} @ {}", data.url, visit.client)}</a>
                    </div>
                }.into_view()
        }))
    }
}

#[derive(Deserialize)]
pub struct OGData {
    pub title: Option<String>,
    pub image: Option<Url>,
    pub url: Url,
    pub description: Option<String>,
    pub site_name: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct WebsiteVisit {
    client: String,
    website: Url,
}
