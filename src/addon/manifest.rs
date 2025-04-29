use super::catalog::{Catalog, CatalogType};
use anyhow::anyhow;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    id: String,
    version: String,
    name: String,
    description: String,
    logo: String,
    resources: Vec<Resource>,
    types: Vec<CatalogType>,
    catalogs: Vec<Catalog>,
}

impl Manifest {
    pub async fn build(config: &str) -> Result<Self> {
        // let catalogs = Catalog::export().await;
        let catalogs = Catalog::from_config(config)?;
        let catalog_types: Vec<CatalogType> = catalogs
            .iter()
            .map(|x| x.catalog_type)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let resources = {
            if !catalogs.is_empty() {
                vec![Resource::Catalog]
            } else {
                vec![]
            }
        };
        Ok(Self {
            id: "com.dynamic.catalogs".to_string(),
            version: "0.0.1".to_string(),
            name: "Dynamic Catalogs".to_string(),
            description: "Dynamic Catalogs".to_string(),
            logo: "logo.png".to_string(),
            resources,
            types: catalog_types,
            catalogs,
        })
    }
}

#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Resource {
    Catalog,
}
