use crate::{
    addon::catalog::{CatalogMeta, CatalogPathOptions, CatalogResponse, CatalogType},
    globals::{Environment, GlobalClient},
};
use anyhow::{anyhow, Context, Result};
use axum::http::HeaderMap;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Url;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{self, Display};

#[derive(Debug, Serialize, Deserialize)]
pub struct TraktCatalog {
    endpoint: TraktEndpoint,
    pagination: Option<TraktPagination>,
    extended_info: bool,
    list_id: Option<String>,
    catalog_type: CatalogType,
    genre: Option<String>,
}

impl TraktCatalog {
    pub fn query(endpoint: TraktEndpoint, catalog_type: CatalogType) -> TraktCatalog {
        Self {
            endpoint,
            pagination: None,
            extended_info: false,
            list_id: None,
            catalog_type,
            genre: None,
        }
    }

    pub fn pagination(&mut self, current_page: i32, items_per_page: i32) -> &mut Self {
        let pagination_details = TraktPagination::new(current_page, items_per_page);
        self.pagination = Some(pagination_details);
        self
    }

    pub fn extended_info(&mut self) -> &mut Self {
        self.extended_info = true;
        self
    }

    pub fn list_id(&mut self, id: &str) -> &mut Self {
        self.list_id = Some(id.to_string());
        self
    }

    pub fn genre(&mut self, genre: &str) -> &mut Self {
        self.genre = Some(genre.to_string());
        self
    }

    pub fn as_b64(&self) -> Result<String> {
        let output_catalog_str = serde_json::to_string(self).map_err(|e| {
            anyhow!(
                "as_b64: Unable to convert TraktCatalog to string: {}",
                e.to_string()
            )
        })?;
        let mut output_string = STANDARD.encode(&output_catalog_str);
        output_string.push_str("-trakt");

        Ok(output_string)
    }

    pub async fn from_catalog_path(path_options: &CatalogPathOptions) -> Result<Value> {
        let output_catalog_str_decoded =
            STANDARD.decode(&path_options.catalog_id).map_err(|e| {
                anyhow!(
                    "from_b64: Error decoding TraktCatalog from b64 to a vec: {}",
                    e.to_string()
                )
            })?;

        let output_catalog_str_decoded_json = String::from_utf8(output_catalog_str_decoded)
            .map_err(|e| {
                anyhow!(
                    "from_b64: Error converting decoded b64 value to json string: {}",
                    e.to_string()
                )
            })?;

        let mut output_catalog: TraktCatalog =
            serde_json::from_str(output_catalog_str_decoded_json.as_str()).map_err(|e| {
                anyhow!(
                    "from_b64: Error converting decoded json string to TraktCatalog struct: {}",
                    e.to_string()
                )
            })?;

        output_catalog.add_catalog_path_options(path_options);

        let trakt_response = output_catalog.build().await.map_err(|e| {
            anyhow!(
                "Unable to build CatalogResponse from Trakt catalog query: {}",
                e.to_string()
            )
        })?;

        if let TraktReponse::CatalogResponse(catalog_response) = trakt_response {
            let output_value = serde_json::to_value(catalog_response)
                .context("Unable to convert TraktResponse of type CatalogResponse to JSON value")?;
            Ok(output_value)
        } else {
            Err(anyhow!(
                "Expected Trakt Catalog Response and did not find it"
            ))
        }
    }

    pub fn add_catalog_path_options(
        &mut self,
        catalog_path_options: &CatalogPathOptions,
    ) -> &mut Self {
        if let Some(genre) = &catalog_path_options.genre {
            self.genre(genre.as_str());
        }
        self.pagination(
            catalog_path_options.pagination.page,
            catalog_path_options.pagination.page_size,
        );
        self
    }

    pub async fn build(&self) -> Result<TraktReponse> {
        let env = Environment::get().context("Unable to get global Environment for Trakt query")?;
        let client = GlobalClient::get()?;

        let mut headers = HeaderMap::new();
        // Required Trakt API headers
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("trakt-api-key", env.trakt_client_id.parse()?);
        headers.insert("trakt-api-version", "2".parse()?);

        let mut url = Url::parse("https://api.trakt.tv")?;

        // Convert catalog type to valid value for Trakt API
        let trakt_catalog_type = match self.catalog_type {
            CatalogType::Movie => "movies",
            CatalogType::Series => "shows",
            _ => return Err(anyhow!("Invalid Trakt Catalog Type")),
        };

        // Append query string based on query type
        let endpoint_path_segments = match self.endpoint {
            TraktEndpoint::TrendingMovies => Ok(vec!["movies", "trending"]),
            TraktEndpoint::List => {
                if let Some(list_id) = &self.list_id {
                    Ok(vec!["lists", list_id.as_str(), "items", trakt_catalog_type])
                } else {
                    Err(anyhow!("No list provided in Trakt List endpoint"))
                }
            }
            TraktEndpoint::Genres => Ok(vec!["genres", trakt_catalog_type]),
        }?;

        url.path_segments_mut()
            .map_err(|e| anyhow!("Cannot be base URL: {:#?}", e))?
            .extend(endpoint_path_segments);

        // Add info level
        if self.extended_info {
            url.query_pairs_mut().append_pair("extended", "full");
        }

        // Add pagination details to query string if provided
        if let Some(pagination_details) = &self.pagination {
            url.query_pairs_mut()
                .append_pair("page", &pagination_details.current_page.to_string())
                .append_pair("limit", &pagination_details.items_per_page.to_string());
        }

        let request = client.get(url).headers(headers).build()?;

        println!("Final URL: {}", request.url());

        let response = client.execute(request).await?;

        let json: Value = response.json().await.map_err(|e| {
            anyhow!(
                "Unable to convert TraktAPI response to json: {}",
                e.to_string()
            )
        })?;

        let output = self
            .endpoint
            .parse_output(json)
            .map_err(|e| anyhow!("Unable to parse output from Trakt API: {}", e.to_string()))?;

        Ok(output)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TraktReponse {
    CatalogResponse(CatalogResponse),
    Genres(Vec<TraktGenre>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TraktEndpoint {
    TrendingMovies,
    List,
    Genres,
}

impl TraktEndpoint {
    fn parse_output(&self, data: Value) -> Result<TraktReponse> {
        match self {
            TraktEndpoint::TrendingMovies => {
                Ok(TraktReponse::CatalogResponse(CatalogResponse::new_empty()))
            }
            TraktEndpoint::List => {
                // Initialize return object
                let mut catalog_response = CatalogResponse::new_empty();

                let data_array = data.as_array().ok_or_else(|| {
                    anyhow!("Parsing Trakt Endpoint: {:#?}: Expected JSON array", &self)
                })?;

                // Map through each member of the json array
                for entry in data_array.iter() {
                    // Required meta
                    let input_type = match entry.get("type").and_then(|t| t.as_str()) {
                        Some(t) => t,
                        None => {
                            return Err(anyhow!("Parsing Trakt Endpoint ({:#?}): Missing or invalid 'key' in entry: {:?}",&self, entry));
                        }
                    };
                    let meta_type = if input_type == "show" {
                        "series"
                    } else {
                        input_type
                    };

                    let inner_entry = match entry.get(input_type) {
                        Some(inner) => inner,
                        None => {
                            // Handle missing key in entry
                            return Err(anyhow!(
                                "Missing key '{}' in entry: {:?}",
                                input_type,
                                entry
                            ));
                        }
                    };

                    let imdb_id = match inner_entry
                        .get("ids")
                        .and_then(|ids| ids.get("imdb"))
                        .and_then(|id| id.as_str())
                    {
                        Some(id) => id,
                        None => {
                            return Err(anyhow!(
                                "Missing or invalid 'imdb' ID in entry: {:?}",
                                entry
                            ));
                        }
                    };

                    let title = match inner_entry.get("title").and_then(|t| t.as_str()) {
                        Some(t) => t,
                        None => {
                            return Err(anyhow!(
                                "Missing or invalid 'title' in entry: {:?}",
                                entry
                            ));
                        }
                    };

                    // Final Meta struct
                    let mut catalog_item = CatalogMeta::new(imdb_id, title, meta_type);
                    // Optional Meta

                    let genres = extract_genres(inner_entry)
                        .map_err(|e| anyhow!("Unable to extract genres: {}", e))?;

                    catalog_item.genres(genres);

                    catalog_response.metas.push(catalog_item);
                }
                Ok(TraktReponse::CatalogResponse(catalog_response))
            }
            TraktEndpoint::Genres => {
                let genres: Vec<TraktGenre> = serde_json::from_value(data).map_err(|e| {
                    anyhow!(
                        "Unable to convert TraktGenre from JSON value to struct: {}",
                        e
                    )
                })?;
                Ok(TraktReponse::Genres(genres))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraktPagination {
    current_page: i32,
    items_per_page: i32,
}

impl TraktPagination {
    fn new(current_page: i32, items_per_page: i32) -> Self {
        Self {
            current_page,
            items_per_page,
        }
    }
}

impl Display for TraktPagination {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "?page={}&limit={}",
            self.current_page, self.items_per_page
        )
    }
}

pub async fn get_trakt_list_id(url: &str) -> Result<String> {
    let client = GlobalClient::get()?;
    let response = client.get(url).send().await?.text().await?;
    let document = Html::parse_document(&response);
    let selector = Selector::parse(r#"input[id="list-id"]"#)
        .map_err(|e| anyhow!("Failed to parse selector: {:?}", e))?;
    let mut list_id = String::new();

    if let Some(element) = document.select(&selector).next() {
        if let Some(value) = element.value().attr("value") {
            list_id.push_str(value);
        }
    }

    match &list_id.is_empty() {
        true => Err(anyhow!("Trakt List ID not found")),
        false => Ok(list_id),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraktGenre {
    pub name: String,
    pub slug: String,
}

fn extract_genres(inner_entry: &Value) -> Result<Vec<String>> {
    inner_entry
        .get("genres")
        .context("Failed to get 'genres' field from entry")?
        .as_array()
        .context("'genres' field is not an array")?
        .iter()
        .map(|genre| {
            genre
                .as_str()
                .context("Expected each genre to be a string")
                .map(|s| s.to_string())
        })
        .collect()
}
