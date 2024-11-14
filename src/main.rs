use addon::catalog::{CatalogPathOptions, CatalogSource, CatalogType};
use addon::Addon;
use anyhow::{Context, Result};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, Router};
use globals::set_globals;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use trakt::{get_trakt_list_id, TraktCatalog, TraktEndpoint, TraktResponse};

mod addon;
mod globals;
mod trakt;

#[tokio::main]
async fn main() -> Result<()> {
    // Set relevant environment variables and make available globally
    set_globals()?;

    let cors = CorsLayer::new().allow_origin(Any);

    // ServeDir for /dashboard and fallback to index.html
    let dashboard_service =
        ServeDir::new("dist").fallback(ServeFile::new(PathBuf::from("dist/index.html")));

    let app = Router::new()
        .route("/health", get(health))
        .route("/example-trakt", get(example_trakt))
        .route("/trakt/extract-list-id", get(trakt_list_id))
        .route("/trakt-genres", get(trakt_genres))
        .route("/:config/manifest.json", get(manifest))
        .route("/:config/catalog/:type/*catalog_path", get(catalog))
        .nest_service("/:config/configure", dashboard_service)
        .layer(cors);

    let address = "127.0.0.1:8080";

    println!("Server listening at {}", address);

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .context("Unable to bind TCP Listener")?;
    axum::serve(listener, app)
        .await
        .context("Unable to serve axum server")?;
    Ok(())
}

async fn health() -> &'static str {
    "Server is up!"
}

async fn example_trakt() -> String {
    let mut catalog = TraktCatalog::query(TraktEndpoint::List, CatalogType::Movie);
    catalog.pagination(1, 500);
    catalog.list_id("20764770");
    catalog.extended_info();
    println!("catalog: {:#?}", catalog);

    let res = catalog.build().await.unwrap();
    if let TraktResponse::CatalogResponse(cat) = res {
        println!("Values given back: {}", cat.metas.len());
    }

    // catalog.as_b64().unwrap();
    String::from("TODO: REPLACE THIS")
}

async fn trakt_genres() -> Result<impl IntoResponse, (StatusCode, String)> {
    let query = TraktCatalog::query(TraktEndpoint::Genres, CatalogType::Movie)
        .build()
        .await
        .unwrap();
    if let TraktResponse::Genres(genres) = query {
        let genres_json = json!(genres);
        Ok((StatusCode::OK, axum::response::Json(genres_json)))
    } else {
        let genres_json = json!({"genres": []});
        Ok((StatusCode::OK, axum::response::Json(genres_json)))
    }
}

async fn manifest(Path(_config): Path<String>) -> impl IntoResponse {
    let addon = Addon::build().await;

    let manifest = json!(addon.manifest);
    (StatusCode::OK, axum::response::Json(manifest))
}

async fn catalog(
    Path((config, catalog_type, catalog_path)): Path<(String, CatalogType, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Ensure the path ends with `.json`
    if !catalog_path.ends_with(".json") {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from("API expects GET request for JSON file. No valid extension provided."),
        ));
    }

    // Extract path options requested by Stremio (i.e. genre, pagination)
    let catalog_path_options = CatalogPathOptions::from_path(&catalog_path).map_err(|e| {
        let error_message = format!("Unable to parse CatalogPathOptions: {}", e);
        ((StatusCode::BAD_REQUEST), error_message)
    })?;

    match catalog_path_options.source {
        CatalogSource::Trakt => {
            let response = TraktCatalog::from_catalog_path(&catalog_path_options)
                .await
                .map_err(|e| {
                    let error_message = format!(
                        "Unable to build TraktCatalog with provided Catalog Path: {}",
                        e
                    );
                    ((StatusCode::BAD_REQUEST), error_message)
                })?;

            Ok((StatusCode::OK, axum::response::Json(response)))
        }
    }
}

async fn trakt_list_id(
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut list_id = String::new();
    if let Some(url) = params.get("url") {
        let scraped_id = get_trakt_list_id(url)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        list_id.push_str(&scraped_id);
    } else {
        return Err(StatusCode::NO_CONTENT);
    }
    let id_json = json!({"id": list_id});
    Ok((StatusCode::OK, axum::response::Json(id_json)))
}
