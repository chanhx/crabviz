use {
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub servers: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            servers: [("rust".into(), "path/to/lsp_server".into())]
                .into_iter()
                .collect(),
        }
    }
}
