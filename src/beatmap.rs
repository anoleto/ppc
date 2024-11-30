use std::path::PathBuf;
use tokio::fs::{self, create_dir_all};
use reqwest;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BeatmapCacheError {
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct BeatmapCache {
    cache_dir: PathBuf,
}

impl BeatmapCache {
    pub fn new() -> Self {
        let cache_dir = PathBuf::from(".data/beatmaps");
        Self { cache_dir }
    }

    pub async fn ensure_cache_exists(&self) -> Result<(), BeatmapCacheError> {
        create_dir_all(&self.cache_dir).await?;
        Ok(())
    }

    pub fn get_beatmap_path(&self, beatmap_id: u64) -> PathBuf {
        self.cache_dir.join(format!("{}.osu", beatmap_id))
    }

    pub async fn get_or_download_beatmap(&self, beatmap_id: u64) -> Result<String, BeatmapCacheError> {
        let beatmap_path = self.get_beatmap_path(beatmap_id);

        if beatmap_path.exists() {
            return Ok(fs::read_to_string(&beatmap_path).await?);
        }

        let url = format!("https://osu.ppy.sh/osu/{}", beatmap_id);
        let client = reqwest::Client::new();
        let beatmap_content = client.get(&url)
            .send()
            .await?
            .text()
            .await?;

        if let Some(parent) = beatmap_path.parent() {
            create_dir_all(parent).await?;
        }

        fs::write(&beatmap_path, &beatmap_content).await?;

        Ok(beatmap_content)
    }
}