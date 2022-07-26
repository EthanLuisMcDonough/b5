use lazy_static::lazy_static;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Webmaster {
    pub email: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub title: String,
    pub url: String,
    pub description: String,
    pub webmaster: Webmaster,
    pub taglines: Vec<String>,
    pub preview_size: usize,
    pub page_size: u64,
    pub rss_size: u64,
}

impl Config {
    pub fn tagline(&self) -> Option<&String> {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.taglines.choose(&mut rng)
    }
}

lazy_static! {
    pub static ref CONFIG: Config = {
        let raw_config =
            std::fs::read_to_string("./config.json").expect("Could not read config.json");
        serde_json::from_str::<Config>(&raw_config).expect("Could not parse config.json")
    };
}
