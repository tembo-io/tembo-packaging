use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TrunkProject {
    pub name: String,
    pub repository_link: String,
    pub version: String,
    pub postgres_versions: Vec<i64>,
    pub extensions: Vec<Extension>,
    pub downloads: Vec<Download>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Extension {
    pub extension_name: String,
    pub version: String,
    pub dependencies_extension_names: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct LoadableLibrary {
    pub library_name: String,
    pub requires_restart: bool,
    pub priority: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Download {
    pub link: String,
    pub pg_version: u8,
    pub platform: String,
    pub sha256: String,
}
