use std::{env, fs};

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CatalogSource {
    Trakt,
}

// First few types for building the catalog for the addons struct
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CatalogType {
    Movie,
    Series,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extra {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Vec<String>>,
    #[serde(rename = "isRequired")]
    is_required: bool,
}

impl Extra {
    fn new(name: &str, options: Option<Vec<String>>, is_required: bool) -> Self {
        Self {
            name: name.to_string(),
            options,
            is_required,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Catalog {
    id: String,
    #[serde(rename = "type")]
    pub catalog_type: CatalogType,
    name: String,
    extra: Vec<Extra>,
}

impl Catalog {
    pub async fn export() -> Vec<Self> {
        // TODO: Make this build dynamically from the user config
        let catalog1 = Self {
            name: "Netflix Movies".to_string(),
            id: "eyJlbmRwb2ludCI6Ikxpc3QiLCJwYWdpbmF0aW9uIjpudWxsLCJleHRlbmRlZF9pbmZvIjp0cnVlLCJsaXN0X2lkIjoiMjA3NjQ3NzAiLCJjYXRhbG9nX3R5cGUiOiJtb3ZpZSJ9-trakt".to_string(),
            catalog_type: CatalogType::Movie,
            extra: vec![Extra::new("skip", None, false)],
        };

        let catalog2 = Self {
            name: "Netflix TV Shows".to_string(),
            id: "eyJlbmRwb2ludCI6Ikxpc3QiLCJwYWdpbmF0aW9uIjpudWxsLCJleHRlbmRlZF9pbmZvIjp0cnVlLCJsaXN0X2lkIjoiMjA3NjQ0NzEiLCJjYXRhbG9nX3R5cGUiOiJzZXJpZXMifQ==-trakt".to_string(),
            catalog_type: CatalogType::Series,
            extra: vec![Extra::new("skip", None, false)],
        };

        let catalog3 = Self {
            name: "Broken Catalog YAY".to_string(),
            id: "elkajsdfyJlbmRwb2ludCI6Ikxpc3QiLCJwYWdpbmF0aW9uIjpudWxsLCJleHRlbmRlZF9pbmZvIjp0cnVlLCJsaXN0X2lkIjoiMjA3NjQ0NzEiLCJjYXRhbG9nX3R5cGUiOiJzZXJpZXMifQ==-trakt".to_string(),
            catalog_type: CatalogType::Series,
            extra: vec![Extra::new("skip", None, false)],
        };

        let sample_catalog_list = vec![catalog1, catalog2, catalog3];

        let catalog_list_json = serde_json::to_string_pretty(&sample_catalog_list).unwrap();

        let mut exe_path = env::current_exe().unwrap();
        exe_path.pop();
        exe_path.push("manifest.json");
        fs::write(&exe_path, catalog_list_json).unwrap();
        println!("Generated manifest.json file at: {:?}", exe_path);

        sample_catalog_list
    }

    pub fn from_config(config: &str) -> Result<Vec<Self>> {
        let config_decoded = STANDARD.decode(config).map_err(|e| {
            anyhow!(
                "from_b64: Error decoding 'Catalogs List' from b64 to a vec: {}",
                e.to_string()
            )
        })?;

        let config_decoded_str = String::from_utf8(config_decoded).map_err(|e| {
            anyhow!(
                "from_b64: Error converting decoded b64 value to json string: {}",
                e.to_string()
            )
        })?;

        let catalogs_from_config: Vec<Self> = serde_json::from_str(config_decoded_str.as_str())
            .map_err(|e| {
                anyhow!(
                    "from_b64: Error converting decoded json string to TraktCatalog struct: {}",
                    e.to_string()
                )
            })?;

        Ok(catalogs_from_config)
    }
}

// Next few types for creating response catalog to send to Stremio
#[derive(Debug, Serialize, Deserialize)]
pub struct CatalogResponse {
    pub metas: Vec<CatalogMeta>,
}

impl CatalogResponse {
    pub fn new_empty() -> Self {
        Self { metas: Vec::new() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultVideoID {
    pub default_video_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trailer {
    pub source: String,
    #[serde(rename = "type")]
    pub trailer_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogMeta {
    #[serde(rename = "type")]
    pub catalog_type: CatalogType,
    pub id: String,
    pub name: String,
    pub poster: Option<String>,
    pub background: Option<String>,
    pub genres: Option<Vec<String>>,
    pub release_info: Option<String>,
    pub description: Option<String>,
    pub behavior_hints: Option<DefaultVideoID>,
    pub trailer: Option<Trailer>,
    pub logo: Option<String>,
    pub runtime: Option<String>,
}

// Following types used for parsing incoming requests from Stremio to the API
#[derive(Debug)]
pub struct CatalogRequestParams {
    pub catalog_id: String,
    pub pagination: PaginationDetails,
    pub genre: Option<String>,
    pub source: CatalogSource,
}

impl CatalogRequestParams {
    pub fn from_path(catalog_path: &str) -> Result<Self> {
        // four scenarios for the catalog_path
        // normal request:           /:config/catalog/:catalog_type/catalog_id.json
        // with pagination:          /:config/catalog/:catalog_type/catalog_id/skip=200.json
        // with genres:              /:config/catalog/:catalog_type/catalog_id/genre=Adventure.json
        // with genres + pagination: /:config/catalog/:catalog_type/catalog_id/skip=43&genre=2024.json

        let catalog_path_segments: Vec<&str> = catalog_path
            .strip_suffix(".json")
            .context("Unable to strip .json suffix from catalog_path")?
            .split("/")
            .collect();

        const PAGE_SIZE: i32 = 100;
        let mut skip = None;
        let mut genre = None;

        // Separate source and catalog_id from path
        let catalog_id_and_source: Vec<String> = catalog_path_segments[0]
            .split("-")
            .map(|id| id.to_string())
            .collect();

        let catalog_source = match catalog_id_and_source[1].as_str() {
            "trakt" => Ok(CatalogSource::Trakt),
            _ => Err(anyhow!("Unable to resolve catalog source to valid value")),
        }?;

        match &catalog_path_segments.len() {
            1 => Ok(CatalogRequestParams {
                catalog_id: catalog_id_and_source[0].clone(),
                pagination: PaginationDetails {
                    page: 1,
                    page_size: PAGE_SIZE,
                },
                genre: None,
                source: catalog_source,
            }),
            2 => {
                let catalog_id = catalog_id_and_source[0].clone();
                let other_catalog_params: Vec<String> = catalog_path_segments[1]
                    .split("&")
                    .map(|path| path.to_string())
                    .collect();

                for param in &other_catalog_params {
                    let parts: Vec<&str> = param.split("=").collect();
                    if parts.len() == 2 {
                        match parts[0] {
                            "skip" => {
                                skip =
                                    Some(parts[1].parse::<i32>().map_err(|e| {
                                        anyhow!("Unable to parse skip value: {}", e)
                                    })?)
                            }
                            "genre" => genre = Some(parts[1].to_string()),
                            _ => {}
                        }
                    }
                }

                let page = if let Some(skip_value) = skip {
                    skip_value / PAGE_SIZE + 1
                } else {
                    1
                };

                Ok(CatalogRequestParams {
                    catalog_id,
                    pagination: PaginationDetails {
                        page,
                        page_size: PAGE_SIZE,
                    },
                    genre,
                    source: catalog_source,
                })
            }
            _ => Err(anyhow!("Incorrect catalog path options provided")),
        }
    }
}

#[derive(Debug)]
pub struct PaginationDetails {
    pub page: i32,
    pub page_size: i32,
}
