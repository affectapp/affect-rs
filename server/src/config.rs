use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub postgres: PostgresConfig,
    pub firebase: FirebaseConfig,
}

#[derive(Deserialize)]
pub struct PostgresConfig {
    pub uri: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FirebaseConfig {
    pub gwk_url: String,
    pub project_id: String,
}
