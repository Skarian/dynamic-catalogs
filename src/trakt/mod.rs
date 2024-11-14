use crate::{
    addon::catalog::{
        CatalogMeta, CatalogPathOptions, CatalogResponse, CatalogType, DefaultVideoID,
        PaginationPath, Trailer,
    },
    globals::{Environment, GlobalClient},
};
use anyhow::{anyhow, Context, Result};
use api::TraktItem;
use axum::http::HeaderMap;
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Url;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::fmt::{self, Display};

pub mod api;

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

        if let TraktResponse::CatalogResponse(catalog_response) = trakt_response {
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
        // When the endpoint is TraktEndpoint::List and a genre is selected, since Trakt's API does
        // not allow sorting via parameters, instead 500 items are pulled on the first page's
        // requests. Any additional page requests are set to empty responses in the build function
        // to avoid sending duplicative catalogs
        let pagination = {
            if let TraktEndpoint::List = self.endpoint {
                if catalog_path_options.genre.is_some() {
                    PaginationPath {
                        page: catalog_path_options.pagination.page,
                        page_size: 500,
                    }
                } else {
                    PaginationPath {
                        page: catalog_path_options.pagination.page,
                        page_size: catalog_path_options.pagination.page_size,
                    }
                }
            } else {
                PaginationPath {
                    page: catalog_path_options.pagination.page,
                    page_size: catalog_path_options.pagination.page_size,
                }
            }
        };
        if let Some(genre) = &catalog_path_options.genre {
            self.genre(genre.as_str());
        }
        self.pagination(pagination.page, pagination.page_size);
        self
    }

    pub async fn build(&self) -> Result<TraktResponse> {
        let env = Environment::get().context("Unable to get global Environment for Trakt query")?;
        let client = GlobalClient::get()?;

        let mut headers = HeaderMap::new();
        // Required Trakt API headers
        headers.insert("Content-Type", "application/json".parse()?);
        headers.insert("trakt-api-key", env.trakt_client_id.parse()?);
        headers.insert("trakt-api-version", "2".parse()?);

        let mut url = Url::parse("https://api.trakt.tv")?;

        // Convert catalog type to valid string for Trakt API
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
            .parse_output(json, &self.genre)
            .map_err(|e| anyhow!("Unable to parse output from Trakt API: {}", e.to_string()))?;

        // Print the number of meta items returned
        if let TraktResponse::CatalogResponse(cat) = &output {
            let count = cat.metas.len();
            println!("Returned {} meta objects", count);
        }

        // This is a continuation of the logic in self::add_catalog_path_options.
        // If additional pages are being requested when its on the List endpoint and there is a genre (sorting)
        // set, send an empty response, first response had 500 to compensate
        if matches!(self.endpoint, TraktEndpoint::List)
            && self.genre.is_some()
            && self
                .pagination
                .as_ref()
                .map_or(false, |p| p.current_page > 1)
        {
            let empty_response = TraktResponse::CatalogResponse(CatalogResponse::new_empty());
            return Ok(empty_response);
        }

        Ok(output)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TraktResponse {
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
    fn parse_output(&self, data: Value, genre: &Option<String>) -> Result<TraktResponse> {
        match self {
            TraktEndpoint::TrendingMovies => {
                Ok(TraktResponse::CatalogResponse(CatalogResponse::new_empty()))
            }
            TraktEndpoint::List => {
                let api_data: Vec<TraktItem> = from_value(data.clone())?;

                // Add sorting logic
                let mut new_catalog_response = CatalogResponse::new_empty();

                for entry in &api_data {
                    let (id, title, description, genres, released, youtube, runtime) = match entry {
                        TraktItem::Movie { movie, .. } => (
                            movie.ids.imdb.clone(),
                            movie.title.clone(),
                            movie.overview.clone(),
                            movie.genres.clone(),
                            movie.year,
                            movie.trailer.clone(),
                            movie.runtime,
                        ),
                        TraktItem::Show { show, .. } => (
                            show.ids.imdb.clone(),
                            show.title.clone(),
                            show.overview.clone(),
                            show.genres.clone(),
                            show.year,
                            show.trailer.clone(),
                            show.runtime,
                        ),
                    };

                    let catalog_type = match entry {
                        TraktItem::Movie { .. } => CatalogType::Movie,
                        TraktItem::Show { .. } => CatalogType::Series,
                    };

                    let poster = format!("https://images.metahub.space/poster/medium/{}/img", id);
                    let background =
                        format!("https://images.metahub.space/background/medium/{}/img", id);

                    let logo = format!("https://images.metahub.space/logo/medium/{}/img", id);

                    let runtime_string = runtime.map(|e| format!("{} mins", e));

                    let behavior_hints = DefaultVideoID {
                        default_video_id: id.clone(),
                    };

                    let released_string: Option<String> = released.map(|num| num.to_string());

                    let trailer = match youtube {
                        Some(youtube_link) => {
                            let youtube_code = extract_video_id(&youtube_link);
                            match youtube_code {
                                Ok(code) => {
                                    let trailer_object = Trailer {
                                        source: code.to_string(),
                                        trailer_type: "Trailer".to_string(),
                                    };
                                    Some(trailer_object)
                                }
                                Err(_) => None,
                            }
                        }
                        None => None,
                    };

                    let meta_item = CatalogMeta {
                        id,
                        name: title,
                        catalog_type,
                        genres,
                        release_info: released_string,
                        background: Some(background),
                        poster: Some(poster),
                        description,
                        behavior_hints: Some(behavior_hints),
                        trailer,
                        logo: Some(logo),
                        runtime: runtime_string,
                    };

                    new_catalog_response.metas.push(meta_item);
                }
                Ok(TraktResponse::CatalogResponse(new_catalog_response))
            }
            TraktEndpoint::Genres => {
                let genres: Vec<TraktGenre> = serde_json::from_value(data).map_err(|e| {
                    anyhow!(
                        "Unable to convert TraktGenre from JSON value to struct: {}",
                        e
                    )
                })?;
                Ok(TraktResponse::Genres(genres))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraktPagination {
    pub current_page: i32,
    pub items_per_page: i32,
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

fn extract_video_id(url: &str) -> Result<&str> {
    // Try to find the index of "v=" in the URL
    if let Some(start) = url.find("v=") {
        // Safely get the next 11 characters as the video ID
        return url
            .get(start + 2..start + 13)
            .ok_or_else(|| anyhow!("Failed to extract video ID"));
    }
    Err(anyhow!("URL does not contain a video ID"))
}
