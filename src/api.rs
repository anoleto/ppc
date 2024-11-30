use std::error::Error;
use refx_pp_rs::{Beatmap, BeatmapExt};
use reqwest;
use crate::models::{LeaderboardResponse, PlayerScore, PPCalculationResult, ScoresResponse};
use crate::beatmap::BeatmapCache;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use futures::future::join_all;

#[derive(Clone)]
struct Cache {
    data: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    cache_duration: Duration,
}

impl Cache {
    fn new() -> Self {
        Cache {
            data: Arc::new(Mutex::new(HashMap::new())),
            cache_duration: Duration::new(60, 0),
        }
    }

    fn get(&self, key: &str) -> Option<String> {
        let cache = self.data.lock().unwrap();
        if let Some((response, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.cache_duration {
                println!("Cache hit for key: {}", key);
                return Some(response.clone());
            } else {
                println!("Cache expired for key: {}", key);
            }
        }
        None
    }

    fn set(&self, key: &str, value: String) {
        let mut cache = self.data.lock().unwrap();
        cache.insert(key.to_string(), (value, Instant::now()));
        println!("Cache updated for key: {}", key);
    }
}

async fn fetch_leaderboard(mode: u8, cache: &Cache) -> Result<LeaderboardResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.refx.online/v1/get_leaderboard?mode={}&limit=10",
        mode
    );
    println!("Fetching leaderboard from mode {}, {}", mode, url);

    if let Some(cached_response) = cache.get(&url) {
        println!("Returning cached leaderboard data.");
        let leaderboard: LeaderboardResponse = serde_json::from_str(&cached_response)?;
        return Ok(leaderboard);
    }

    let client = reqwest::Client::new();
    let leaderboard = client.get(&url)
        .send()
        .await?
        .json::<LeaderboardResponse>()
        .await?;

    cache.set(&url, serde_json::to_string(&leaderboard)?);

    println!("Fetched leaderboard successfully.");
    Ok(leaderboard)
}

async fn fetch_player_scores(player_id: u64, mode: u8, cache: &Cache) -> Result<Vec<PlayerScore>, Box<dyn Error>> {
    let url = format!(
        "https://api.refx.online/v1/get_player_scores?id={}&mode={}&scope=best&limit=10",
        player_id, mode
    );
    println!("Fetching scores for player {} in mode {} from {}", player_id, mode, url);

    if let Some(cached_response) = cache.get(&url) {
        println!("Returning cached scores for player {}", player_id);
        let scores_response: ScoresResponse = serde_json::from_str(&cached_response)?;
        return Ok(scores_response.scores);
    }

    let client = reqwest::Client::new();
    let player_scores = client.get(&url)
        .send()
        .await?
        .json::<ScoresResponse>()
        .await?;

    cache.set(&url, serde_json::to_string(&player_scores)?);

    println!("Fetched {} scores for player {}", player_scores.scores.len(), player_id);
    Ok(player_scores.scores)
}

async fn calculate_pp(beatmap_path: &str, score: &PlayerScore, player_name: &str) -> Result<PPCalculationResult, Box<dyn Error>> {
    println!("Calculating PP for player '{}' using beatmap path '{}'", player_name, beatmap_path);

    let map = Beatmap::from_path(beatmap_path)?;

    let original_pp = score.pp;

    let result = map.pp()
        .mods(score.mods)
        .combo(score.max_combo)
        .accuracy(score.acc)
        .n300(score.n300)
        .n100(score.n100)
        .n50(score.n50)
        .n_misses(score.nmiss)
        .calculate();

    let recalculated_pp = result.pp();
    let difference = recalculated_pp - original_pp;

    println!("Calculated PP for player '{}': Original = {}, Recalculated = {}, Difference = {}", player_name, original_pp, recalculated_pp, difference);

    Ok(PPCalculationResult {
        beatmap_id: score.beatmap.id,
        original_pp,
        recalculated_pp,
        difference,
        mods: score.mods,
        version: 0,
    })
}

pub async fn calculate_pp_relax(beatmap_path: &str, score: &PlayerScore, player_name: &str) -> Result<PPCalculationResult, Box<dyn Error>> {
    println!(
        "Calculating RX PP for player '{}' using beatmap path '{}'",
        player_name, beatmap_path
    );

    let map = Beatmap::from_path(beatmap_path)?;

    let original_pp = score.pp;

    let result = refx_pp_rs::osu_2019::OsuPP::new(&map)
        .mods(score.mods)
        .combo(score.max_combo)
        .accuracy(score.acc as f32)
        .n300(score.n300)
        .n100(score.n100)
        .n50(score.n50)
        .misses(score.nmiss)
        .calculate();

    let recalculated_pp = result.pp;
    let difference = recalculated_pp - original_pp;

    println!(
        "Calculated PP for player '{}': Original = {}, Recalculated = {}, Difference = {}",
        player_name, original_pp, recalculated_pp, difference
    );

    Ok(PPCalculationResult {
        beatmap_id: score.beatmap.id,
        original_pp,
        recalculated_pp,
        difference,
        mods: score.mods,
        version: 1,
    })
}

pub async fn calculate_pp_scorev2(beatmap_path: &str, score: &PlayerScore, player_name: &str, relax: bool) -> Result<PPCalculationResult, Box<dyn Error>> {
    println!(
        "Calculating ScoreV2 PP for player '{}' using beatmap path '{}'",
        player_name, beatmap_path
    );

    let map = Beatmap::from_path(beatmap_path)?;

    let original_pp = score.pp;

    let result = refx_pp_rs::osu_2019_scorev2::FxPP::new_from_map(&map)
        .mods(score.mods | if relax { 1 << 7 } else { 0 })
        .combo(score.max_combo)
        .accuracy(score.acc as f32)
        .n300(score.n300)
        .n100(score.n100)
        .n50(score.n50)
        .misses(score.nmiss)
        .calculate();

    let recalculated_pp = result.pp;
    let difference = recalculated_pp - original_pp;

    println!(
        "Calculated PP for player '{}': Original = {}, Recalculated = {}, Difference = {}",
        player_name, original_pp, recalculated_pp, difference
    );

    Ok(PPCalculationResult {
        beatmap_id: score.beatmap.id,
        original_pp,
        recalculated_pp,
        difference,
        mods: score.mods | if relax { 1 << 7 } else { 0 },
        version: 2,
    })
}

pub async fn calculate_pp_now(
    mode: u8, 
    beatmap_cache: &BeatmapCache, 
    version: u8,
    rx: bool,
) -> Result<HashMap<String, Vec<PPCalculationResult>>, Box<dyn Error>> {
    println!("Calculating PP for leaderboard in mode {}", mode);

    let mut pp_results: HashMap<String, Vec<PPCalculationResult>> = HashMap::new();
    let cache = Cache::new();

    println!("Fetching global leaderboard...");
    let leaderboard = fetch_leaderboard(mode, &cache).await?;
    println!("Fetched leaderboard with {} entries.", leaderboard.leaderboard.len());

    let mut tasks = vec![];

    for entry in leaderboard.leaderboard {
        let player_id = entry.player_id;
        let player_name = entry.name.clone();
        let mode = mode;
        let cache = cache.clone();

        let player_task = tokio::spawn(async move {
            let scores = fetch_player_scores(player_id, mode, &cache).await.unwrap_or_default();
            (player_name, scores)
        });

        tasks.push(player_task);
    }

    let player_results = join_all(tasks).await;

    for result in player_results {
        let (player_name, scores) = result.unwrap();
        println!("Fetched {} scores for player '{}'", scores.len(), player_name);

        let mut results: Vec<PPCalculationResult> = Vec::new();

        for score in scores {
            let beatmap_path = beatmap_cache.get_beatmap_path(score.beatmap.id);

            if !beatmap_path.exists() {
                println!("Beatmap {} not cached. Downloading...", score.beatmap.id);
                beatmap_cache.get_or_download_beatmap(score.beatmap.id).await?;
                println!("Downloaded and cached beatmap {}.", score.beatmap.id);
            } else {
                println!("Beatmap {} found in cache.", score.beatmap.id);
            }

            let pp_result = match version {
                0 => calculate_pp(beatmap_path.to_str().unwrap(), &score, &player_name).await?,
                1 => calculate_pp_relax(beatmap_path.to_str().unwrap(), &score, &player_name).await?,
                2 => calculate_pp_scorev2(beatmap_path.to_str().unwrap(), &score, &player_name, rx).await?,
                _ => return Err("Invalid version!".into()),
            };
            
            results.push(pp_result);
        }

        pp_results.insert(player_name, results);
    }

    println!("Finished calculating PP for leaderboard.");
    Ok(pp_results)
}