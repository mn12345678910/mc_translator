use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    #[serde(skip)]
    pub api_key: String,
    pub provider: String,
}

fn main() {
    let config = AppConfig {
        api_key: "secret".to_string(),
        provider: "Gemini".to_string(),
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("{}", json);
}
