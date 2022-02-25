use serde::Deserialize;

#[derive(Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub postgres: PostgresConfig,
    pub firebase: FirebaseConfig,
    pub change: ChangeConfig,
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

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ChangeConfig {
    pub public_key: String,
    pub secret_key: String,
}
