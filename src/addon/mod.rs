use manifest::Manifest;

pub mod catalog;
pub mod manifest;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Addon {
    pub transport_url: String,
    pub transport_name: String,
    pub manifest: Manifest,
    pub flags: Vec<String>,
}

impl Addon {
    pub async fn build() -> Self {
        let manifest = Manifest::build().await;
        Self {
            transport_url: "todo".to_string(),
            transport_name: "todo".to_string(),
            manifest,
            flags: vec!["todo".to_string(), "todo".to_string()],
        }
    }
}
