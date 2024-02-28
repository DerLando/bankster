/// Path to USB on pi: /media/lando/LANDOSTICK
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct Config {
    pub(crate) backup_dir: String,
    pub(crate) assets_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backup_dir: "./".to_string(),
            assets_dir: "./appserver/assets".to_string(),
        }
    }
}
