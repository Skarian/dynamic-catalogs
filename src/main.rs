use addon::catalog::{CatalogRequestParams, CatalogSource, CatalogType};
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
use trakt::{get_trakt_list_id, TraktCatalog};

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
        .route("/:config/manifest.json", get(manifest))
        .nest_service("/:config/configure", dashboard_service)
        .route("/:config/catalog/:type/*stremio_catalog_path", get(catalog))
        // .route("/example-trakt", get(example_trakt))
        .route("/trakt/extract-list-id", get(trakt_list_id))
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

// async fn example_trakt() -> String {
//     let mut catalog = TraktCatalog::query(TraktEndpoint::List, CatalogType::Movie);
//     catalog.pagination(1, 100);
//     catalog.list_id("20764770");
//     println!("catalog: {:#?}", catalog);
//
//     let res = catalog.build().await.unwrap();
//     if let TraktResponse::CatalogResponse(cat) = res {
//         println!("Values given back: {}", cat.metas.len());
//     }
//
//     // catalog.as_b64().unwrap();
//     String::from("TODO: REPLACE THIS")
// }

async fn manifest(Path(config): Path<String>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let addon = Addon::build(&config)
        .await
        .map_err(|e| (StatusCode::OK, e.to_string()))?;

    let manifest = json!(addon.manifest);
    Ok((StatusCode::OK, axum::response::Json(manifest)))
}

async fn catalog(
    Path((config, catalog_type, stremio_catalog_path)): Path<(String, CatalogType, String)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Ensure the path ends with `.json`
    // This will remain true for every request from Stremio
    if !stremio_catalog_path.ends_with(".json") {
        return Err((
            StatusCode::BAD_REQUEST,
            String::from("API expects GET request for JSON file. No valid extension provided."),
        ));
    }

    // Extract path options provided by Stremio (i.e. genre, pagination)
    // We are storing the entire catalog config in the catalog_id provided by Stremio
    let catalog_params = CatalogRequestParams::from_path(&stremio_catalog_path).map_err(|e| {
        let error_message = format!("Unable to parse CatalogPathOptions: {}", e);
        ((StatusCode::BAD_REQUEST), error_message)
    })?;

    // Build catalog from parsed params based on query source, each CatalogParams vary by source
    let response = match catalog_params.source {
        CatalogSource::Trakt => {
            let response = TraktCatalog::from_catalog_params(&catalog_params)
                .await
                .map_err(|e| {
                    let error_message = format!(
                        "Unable to build TraktCatalog with provided Catalog Path: {}",
                        e
                    );
                    ((StatusCode::BAD_REQUEST), error_message)
                })?;

            (StatusCode::OK, axum::response::Json(response)).into_response()
        }
    };

    Ok(response)
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
