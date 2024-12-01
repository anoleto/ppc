use std::error::Error;
use std::collections::HashMap;
use std::env;

use reqwest;
use futures::future::join_all;

use crate::models::{
    LeaderboardResponse, 
    PlayerScore, 
    PPCalculationResult, 
    ScoresResponse
};
use crate::beatmap::BeatmapCache;
use super::cache::Cache;
use crate::calculate::calculate;

async fn fetch_leaderboard(mode: u8, cache: &Cache) -> Result<LeaderboardResponse, Box<dyn Error>> {
    let url = format!(
        "https://api.{}/v1/get_leaderboard?mode={}&limit=10",
        env::var("URL").expect(""), mode
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
        "https://api.{}/v1/get_player_scores?id={}&mode={}&scope=best&limit=10",
        env::var("URL").expect(""), player_id, mode
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

pub async fn calculate_pp_now(
    mode: u8, 
    beatmap_cache: &BeatmapCache, 
    version: u8,
    rx: bool,
    branch: u8,
) -> Result<HashMap<String, Vec<PPCalculationResult>>, Box<dyn Error>> {
    println!("Calculating PP for leaderboard in mode {}", mode);

    let mut pp_results: HashMap<String, Vec<PPCalculationResult>> = HashMap::new();
    let cache = Cache::new(60);

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

            // LMFAOO IM CRYING
            let pp_result = match branch {
                0 => match version {
                    0 => calculate::calculate_pp_vn(beatmap_path.to_str().unwrap(), &score, &player_name).await?,
                    1 => calculate::calculate_pp_relax(beatmap_path.to_str().unwrap(), &score, &player_name).await?,
                    2 => calculate::calculate_pp_scorev2(beatmap_path.to_str().unwrap(), &score, &player_name, rx).await?,
                    _ => return Err("Invalid version!".into()),
                },
                1 => match version {
                    0 => calculate::calculate_pp_vn_lgt(beatmap_path.to_str().unwrap(), &score, &player_name).await?,
                    1 => calculate::calculate_pp_relax_lgt(beatmap_path.to_str().unwrap(), &score, &player_name).await?,
                    2 => calculate::calculate_pp_scorev2_lgt(beatmap_path.to_str().unwrap(), &score, &player_name, rx).await?,
                    _ => return Err("Invalid version!".into()),
                },
                _ => return Err("Invalid branch!".into()),
            };
            
            results.push(pp_result);
        }

        pp_results.insert(player_name, results);
    }

    println!("Finished calculating PP for leaderboard.");
    Ok(pp_results)
}