use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub port_env_var: Option<String>,
    pub postgres: PostgresConfig,
    pub firebase: FirebaseConfig,
    pub change: ChangeConfig,
    pub plaid: PlaidConfig,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
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

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PlaidConfig {
    pub client_id: String,
    pub secret_key: String,
    pub env: String,
}
