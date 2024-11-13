use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::trakt::{TraktCatalog, TraktEndpoint, TraktReponse};

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
    Unknown,
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
    pub async fn build() -> Vec<Self> {
        let mut trakt_movies_genres: Vec<String> = Vec::new();
        let mut trakt_series_genres: Vec<String> = Vec::new();
        let trakt_sort_values: Vec<String> = [
            "Trending Now",
            "New Releases",
            "A-Z",
            "Short & Sweet",
            "Top Rated",
            "Recently Watched",
            "Fan Favorites",
        ]
        .iter()
        .map(|sort| sort.to_string())
        .collect();

        // Get Trakt Movie Genres
        let trakt_movie_genres_query =
            TraktCatalog::query(TraktEndpoint::Genres, CatalogType::Movie)
                .build()
                .await
                .unwrap();

        if let TraktReponse::Genres(genres) = trakt_movie_genres_query {
            trakt_movies_genres = genres.into_iter().map(|genre| genre.slug).collect()
        }

        // Get Trakt Series Genres
        let trakt_series_genres_query =
            TraktCatalog::query(TraktEndpoint::Genres, CatalogType::Series)
                .build()
                .await
                .unwrap();

        if let TraktReponse::Genres(genres) = trakt_series_genres_query {
            trakt_series_genres = genres.into_iter().map(|genre| genre.slug).collect()
        }

        let catalog1 = Self {
            name: "Netflix Movies".to_string(),
            id: "eyJlbmRwb2ludCI6Ikxpc3QiLCJwYWdpbmF0aW9uIjpudWxsLCJleHRlbmRlZF9pbmZvIjp0cnVlLCJsaXN0X2lkIjoiMjA3NjQ3NzAiLCJjYXRhbG9nX3R5cGUiOiJtb3ZpZSJ9-trakt".to_string(),
            catalog_type: CatalogType::Movie,
            extra: vec![Extra::new("skip", None, false), Extra::new("genre", Some(trakt_sort_values.clone()), false)],
        };

        let catalog2 = Self {
            name: "Netflix TV Shows".to_string(),
            id: "eyJlbmRwb2ludCI6Ikxpc3QiLCJwYWdpbmF0aW9uIjpudWxsLCJleHRlbmRlZF9pbmZvIjp0cnVlLCJsaXN0X2lkIjoiMjA3NjQ0NzEiLCJjYXRhbG9nX3R5cGUiOiJzZXJpZXMifQ==-trakt".to_string(),
            catalog_type: CatalogType::Series,
            extra: vec![Extra::new("skip", None, false), Extra::new("genre",Some(trakt_sort_values), false)],
        };

        vec![catalog1, catalog2]
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
pub struct CatalogMeta {
    #[serde(rename = "type")]
    pub catalog_type: CatalogType,
    id: String,
    name: String,
    poster: String,
    genres: Option<Vec<String>>,
}

impl CatalogMeta {
    pub fn new(id: &str, name: &str, catalog_type: &str) -> Self {
        let catalog_meta_type = match catalog_type.to_lowercase().as_str() {
            "movie" => CatalogType::Movie,
            "show" => CatalogType::Series,
            "series" => CatalogType::Series,
            _ => CatalogType::Unknown,
        };
        Self {
            catalog_type: catalog_meta_type,
            id: id.to_string(),
            name: name.to_string(),
            poster: format!("https://images.metahub.space/poster/medium/{}/img", id),
            genres: None,
        }
    }

    pub fn genres(&mut self, genres: Vec<String>) -> &mut Self {
        self.genres = Some(genres);
        self
    }
}

// Following types used for parsing incoming requests from Stremio to the API
#[derive(Debug)]
pub struct CatalogPathOptions {
    pub catalog_id: String,
    pub pagination: PaginationPath,
    pub genre: Option<String>,
    pub source: CatalogSource,
}

impl CatalogPathOptions {
    pub fn from_path(catalog_path: &str) -> Result<Self> {
        // four scenarios
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
            1 => Ok(CatalogPathOptions {
                catalog_id: catalog_id_and_source[0].clone(),
                pagination: PaginationPath {
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

                Ok(CatalogPathOptions {
                    catalog_id,
                    pagination: PaginationPath {
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
pub struct PaginationPath {
    pub page: i32,
    pub page_size: i32,
}