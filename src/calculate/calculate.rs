
/// this only calculates standard
/// used to calculate reworks and future updates on the
/// std pp system

use std::error::Error;
use crate::models::{PlayerScore, PPCalculationResult};
use crate::calculate::utils::round;

use refx_pp_rs::{Beatmap, BeatmapExt};
use if_servers_legit::{Beatmap as ifLegitBeatmap, BeatmapExt as ifLegitExt};
use live_pp::{Beatmap as livePPBeatmap, BeatmapExt as livePPExt};

#[derive(Clone, Copy)]
pub enum PPCalculationType {

    /// main without cv (cheat value)
    VanillaNoCV,
    RelaxNoCV,
    ScoreV2NoCV { relax: bool },

    /// main with cv (cheat value)
    VanillaCheats,
    RelaxCheats,
    ScoreV2Cheats { relax: bool },

    /// if-servers-legit
    VanillaLegit,
    RelaxLegit,
    ScoreV2Legit { relax: bool },

    /// live pp
    VanillaCheatsLive,
    RelaxCheatsLive,
    ScoreV2CheatsLive { relax: bool },

}

pub async fn calculate_pp(
    beatmap_path: &str,
    score: &PlayerScore,
    player_name: &str,
    calc_type: PPCalculationType,
) -> Result<PPCalculationResult, Box<dyn Error>> {
    println!(
        "Calculating PP for player '{}' using beatmap path '{}'",
        player_name, beatmap_path
    );

    let original_pp = round(score.pp, 2);
    let (recalculated_pp, stars, mods) = match calc_type {

        PPCalculationType::VanillaNoCV => {
            let map = Beatmap::from_path(beatmap_path)?;
            let result = map.pp()
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .n_misses(score.nmiss)
                .calculate();
            (result.pp(), result.stars(), score.mods)
        },

        PPCalculationType::RelaxNoCV => {
            let map = Beatmap::from_path(beatmap_path)?;
            let result = refx_pp_rs::osu_2019::OsuPP::new(&map)
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .calculate();
            (result.pp, result.difficulty.stars, score.mods)
        },

        // shouldnt needed?
        // theres no change here
        // wait nvm there is
        PPCalculationType::ScoreV2NoCV { relax } => {
            let map = Beatmap::from_path(beatmap_path)?;
            let mods = score.mods | if relax { 1 << 7 } else { 0 };
            let result = refx_pp_rs::osu_2019_2::FxPP::new_from_map(&map)
                .mods(mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .calculate();
            (result.pp, result.difficulty.stars, mods)
        },

        PPCalculationType::VanillaCheats => {
            let map = Beatmap::from_path(beatmap_path)?;
            let result = map.pp()
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .n_misses(score.nmiss)
                .ac(score.aim_value)
                .arc(score.ar_value)
                .hdr(score.hdr != 0)
                .calculate();
            (result.pp(), result.stars(), score.mods)
        },

        PPCalculationType::RelaxCheats => {
            let map = Beatmap::from_path(beatmap_path)?;
            let result = refx_pp_rs::osu_2019::OsuPP::new(&map)
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .ac(score.aim_value)
                .arc(score.ar_value)
                .tw(score.twval as usize)
                .cs(score.cs != 0)
                .calculate();
            (result.pp, result.difficulty.stars, score.mods)
        },

        PPCalculationType::ScoreV2Cheats { relax } => {
            let map = Beatmap::from_path(beatmap_path)?;
            let mods = score.mods | if relax { 1 << 7 } else { 0 };
            let result = refx_pp_rs::osu_2019_2::FxPP::new_from_map(&map)
                .mods(mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .calculate();
            (result.pp, result.difficulty.stars, mods)
        },

        PPCalculationType::VanillaLegit => {
            let map = ifLegitBeatmap::from_path(beatmap_path).await?;
            let result = map.pp()
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .n_misses(score.nmiss)
                .calculate();
            (result.pp(), result.stars(), score.mods)
        },

        PPCalculationType::RelaxLegit => {
            let map = ifLegitBeatmap::from_path(beatmap_path).await?;
            let result = if_servers_legit::osu_2019::OsuPP::new(&map)
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .calculate();
            (result.pp, result.difficulty.stars, score.mods)
        },

        PPCalculationType::ScoreV2Legit { relax } => {
            let map = ifLegitBeatmap::from_path(beatmap_path).await?;
            let mods = score.mods | if relax { 1 << 7 } else { 0 };
            let result = if_servers_legit::osu_2019_scorev2::FxPP::new_from_map(&map)
                .mods(mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .calculate();
            (result.pp, result.difficulty.stars, mods)
        },
        
        PPCalculationType::VanillaCheatsLive => {
            let map = livePPBeatmap::from_path(beatmap_path).await?;
            let result = map.pp()
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .n_misses(score.nmiss)
                .ac(score.aim_value)
                .arc(score.ar_value)
                .hdr(score.hdr != 0)
                .calculate();
            (result.pp(), result.stars(), score.mods)
        },

        PPCalculationType::RelaxCheatsLive => {
            let map = livePPBeatmap::from_path(beatmap_path).await?;
            let result = live_pp::osu_2019::OsuPP::new(&map)
                .mods(score.mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .ac(score.aim_value)
                .arc(score.ar_value)
                .tw(score.twval as usize)
                .cs(score.cs != 0)
                .calculate();
            (result.pp, result.difficulty.stars, score.mods)
        },

        PPCalculationType::ScoreV2CheatsLive { relax } => {
            let map = livePPBeatmap::from_path(beatmap_path).await?;
            let mods = score.mods | if relax { 1 << 7 } else { 0 };
            let result = live_pp::osu_2019_2::FxPP::new_from_map(&map)
                .mods(mods)
                .combo(score.max_combo)
                .accuracy(score.acc as f32)
                .n300(score.n300)
                .n100(score.n100)
                .n50(score.n50)
                .misses(score.nmiss)
                .calculate();
            (result.pp, result.difficulty.stars, mods)
        },
    };

    let mut final_pp = round(recalculated_pp, 2);
    let mut final_stars = round(stars, 2);

    // this sometimes happen
    if final_pp.is_infinite() || final_pp.is_nan() {
        println!("Calculated pp is infinite or NaN");
        final_pp = 0.0;
    }

    if final_stars.is_infinite() || final_stars.is_nan() {
        println!("Calculated stars is infinite or NaN");
        final_stars = 0.0;
    }

    let difference = final_pp - original_pp;

    println!(
        "Calculated PP for player '{}': Original = {}, Recalculated = {}, Difference = {}, Stars = {}", 
        player_name, original_pp, final_pp, difference, final_stars
    );

    Ok(PPCalculationResult {
        stars: final_stars,
        beatmap_id: score.beatmap.id,
        original_pp,
        recalculated_pp: final_pp,
        difference,
        mods,
        version: match calc_type {
            PPCalculationType::VanillaNoCV | PPCalculationType::VanillaCheats | 
            PPCalculationType::VanillaLegit | PPCalculationType::VanillaCheatsLive => 0,
            PPCalculationType::RelaxNoCV | PPCalculationType::RelaxCheats | 
            PPCalculationType::RelaxLegit | PPCalculationType::RelaxCheatsLive => 1,
            PPCalculationType::ScoreV2NoCV { .. } | PPCalculationType::ScoreV2Cheats { .. } | 
            PPCalculationType::ScoreV2Legit { .. } | PPCalculationType::ScoreV2CheatsLive { .. } => 2,
        },
    })
}