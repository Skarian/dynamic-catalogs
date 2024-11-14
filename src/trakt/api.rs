use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Ids {
    pub trakt: u32,
    pub slug: String,
    pub imdb: String,
    pub tmdb: Option<u32>,
    pub tvdb: Option<u32>,
    pub tvrage: Option<u32>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Airs {
    pub day: Option<String>,
    pub time: Option<String>,
    pub timezone: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TraktMovie {
    pub title: String,
    pub year: Option<u32>,
    pub ids: Ids,
    pub available_translations: Option<Vec<String>>,
    pub certification: Option<String>,
    pub comment_count: Option<u32>,
    pub country: Option<String>,
    pub genres: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub language: Option<String>,
    pub languages: Option<Vec<String>>,
    pub overview: Option<String>,
    pub rating: Option<f64>,
    pub released: Option<String>, // Date as String
    pub runtime: Option<u32>,
    pub status: Option<String>,
    pub tagline: Option<String>,
    pub trailer: Option<String>,
    pub updated_at: Option<String>, // DateTime as String
    pub votes: Option<u32>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TraktShow {
    pub title: String,
    pub year: Option<u32>,
    pub ids: Ids,
    pub available_translations: Option<Vec<String>>,
    pub certification: Option<String>,
    pub comment_count: Option<u32>,
    pub country: Option<String>,
    pub genres: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub language: Option<String>,
    pub languages: Option<Vec<String>>,
    pub overview: Option<String>,
    pub rating: Option<f64>,
    pub runtime: Option<u32>,
    pub status: Option<String>,
    pub tagline: Option<String>,
    pub trailer: Option<String>,
    pub updated_at: Option<String>, // DateTime as String
    pub votes: Option<u32>,
    // Show-specific fields
    pub aired_episodes: Option<u32>,
    pub airs: Option<Airs>,
    pub first_aired: Option<String>, // DateTime as String
    pub network: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TraktItem {
    #[serde(rename = "movie")]
    Movie {
        id: u32,
        listed_at: String,
        notes: Option<String>,
        rank: u32,
        movie: TraktMovie,
    },
    #[serde(rename = "show")]
    Show {
        id: u32,
        listed_at: String,
        notes: Option<String>,
        rank: u32,
        show: TraktShow,
    },
}
