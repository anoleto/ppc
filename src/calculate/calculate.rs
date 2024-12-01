use std::error::Error;
use crate::models::{PlayerScore, PPCalculationResult};
use crate::calculate::utils::round;
use refx_pp_rs::{Beatmap, BeatmapExt};

pub async fn calculate_pp_vn(beatmap_path: &str, score: &PlayerScore, player_name: &str) -> Result<PPCalculationResult, Box<dyn Error>> {
    println!("Calculating PP for player '{}' using beatmap path '{}'", player_name, beatmap_path);

    let map = Beatmap::from_path(beatmap_path)?;

    let original_pp = round(score.pp, 2);

    let result = map.pp()
        .mods(score.mods)
        .combo(score.max_combo)
        .accuracy(score.acc)
        .n300(score.n300)
        .n100(score.n100)
        .n50(score.n50)
        .n_misses(score.nmiss)
        .calculate();

    let mut recalculated_pp = round(result.pp(), 2);

    if recalculated_pp.is_infinite() || recalculated_pp.is_nan() {
        println!("Calculated pp is infinite or NaN");
        recalculated_pp = 0.0;
    }

    let difference = recalculated_pp - original_pp;

    let mut stars = round(result.stars(), 2);

    if stars.is_infinite() || stars.is_nan() {
        println!("Calculated stars is infinite or NaN");
        stars = 0.0;
    }

    println!(
        "Calculated PP for player '{}': Original = {}, Recalculated = {}, Difference = {}, Stars = {}", 
        player_name, original_pp, recalculated_pp, difference, stars
    );

    Ok(PPCalculationResult {
        stars,
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

    let original_pp = round(score.pp, 2);

    let result = refx_pp_rs::osu_2019::OsuPP::new(&map)
        .mods(score.mods)
        .combo(score.max_combo)
        .accuracy(score.acc as f32)
        .n300(score.n300)
        .n100(score.n100)
        .n50(score.n50)
        .misses(score.nmiss)
        .calculate();

    let mut recalculated_pp = round(result.pp, 2);

    if recalculated_pp.is_infinite() || recalculated_pp.is_nan() {
        println!("Calculated RX PP is infinite or NaN");
        recalculated_pp = 0.0;
    }

    let difference = recalculated_pp - original_pp;

    let mut stars = round(result.difficulty.stars, 2);

    if stars.is_infinite() || stars.is_nan() {
        println!("Calculated RX stars is infinite or NaN");
        stars = 0.0;
    }

    println!(
        "Calculated RX PP for player '{}': Original = {}, Recalculated = {}, Difference = {}, Stars = {}", 
        player_name, original_pp, recalculated_pp, difference, stars
    );

    Ok(PPCalculationResult {
        stars,
        beatmap_id: score.beatmap.id,
        original_pp,
        recalculated_pp,
        difference,
        mods: score.mods,
        version: 1,
    })
}

pub async fn calculate_pp_scorev2(
    beatmap_path: &str, 
    score: &PlayerScore, 
    player_name: &str, 
    relax: bool
) -> Result<PPCalculationResult, Box<dyn Error>> {
    println!(
        "Calculating ScoreV2 PP for player '{}' using beatmap path '{}'",
        player_name, beatmap_path
    );

    let map = Beatmap::from_path(beatmap_path)?;

    let original_pp = round(score.pp, 2);

    let result = refx_pp_rs::osu_2019_scorev2::FxPP::new_from_map(&map)
        .mods(score.mods | if relax { 1 << 7 } else { 0 })
        .combo(score.max_combo)
        .accuracy(score.acc as f32)
        .n300(score.n300)
        .n100(score.n100)
        .n50(score.n50)
        .misses(score.nmiss)
        .calculate();

    let mut recalculated_pp = round(result.pp, 2);

    if recalculated_pp.is_infinite() || recalculated_pp.is_nan() {
        println!("Calculated pp is infinite or NaN");
        recalculated_pp = 0.0;
    }

    let difference = recalculated_pp - original_pp;
    let mut stars = round(result.difficulty.stars, 2);

    if stars.is_infinite() || stars.is_nan() {
        println!("Calculated stars is infinite or NaN");
        stars = 0.0;
    }

    println!(
        "Calculated PP for player '{}': Original = {}, Recalculated = {}, Difference = {}, Stars = {}", 
        player_name, original_pp, recalculated_pp, difference, stars
    );

    Ok(PPCalculationResult {
        stars,
        beatmap_id: score.beatmap.id,
        original_pp,
        recalculated_pp,
        difference,
        mods: score.mods | if relax { 1 << 7 } else { 0 },
        version: 2,
    })
}