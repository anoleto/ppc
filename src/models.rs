use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerScore {
    pub score: u64,
    pub pp: f64,
    pub acc: f64,
    pub max_combo: usize,
    pub mods: u32,
    pub n300: usize,
    pub n100: usize,
    pub n50: usize,
    pub nmiss: usize,
    pub beatmap: BeatmapInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BeatmapInfo {
    pub id: u64,
    pub md5: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeaderboardEntry {
    pub player_id: u64,
    pub name: String,
    pub pp: f64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeaderboardResponse {
    pub status: String,
    pub leaderboard: Vec<LeaderboardEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoresResponse {
    pub status: String,
    pub scores: Vec<PlayerScore>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PPCalculationResult {
    pub beatmap_id: u64,
    pub original_pp: f64,
    pub recalculated_pp: f64,
    pub difference: f64,
    pub mods: u32,
    pub version: u8,
}