mod models;
mod api;
mod beatmap;

use axum::{
    extract::{Query, State},
    routing::get,
    Router,
    response::Json,
    http::StatusCode,
};
use std::error::Error;
use std::collections::HashMap;

use crate::beatmap::BeatmapCache;
use crate::api::calculate_pp_now;

async fn handle_pp_calculation(
    State(beatmap_cache): State<BeatmapCache>,
    Query(params): Query<HashMap<String, String>>
) -> Result<Json<HashMap<String, Vec<models::PPCalculationResult>>>, (StatusCode, String)> {
    let mode = params.get("mode")
        .and_then(|m| m.parse::<u8>().ok())
        .unwrap_or(0);
    let version = params.get("version")
        .and_then(|m| m.parse::<u8>().ok())
        .unwrap_or(0);
    let rx = params.get("rx")
        .and_then(|m| m.parse::<bool>().ok())
        .unwrap_or(false);

    if mode > 8 {
        return Err((
            StatusCode::BAD_REQUEST, 
            "Invalid mode. Must be between 0 and 8.".to_string()
        ));
    }

    if version > 2 {
        return Err((
            StatusCode::BAD_REQUEST, 
            "Invalid version. Must be between 0 and 2.".to_string()
        ));
    }

    match calculate_pp_now(mode, &beatmap_cache, version, rx).await {
        Ok(results) => Ok(Json(results)),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR, 
                "Failed to calculate PP".to_string()
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let beatmap_cache = BeatmapCache::new();

    beatmap_cache.ensure_cache_exists().await?;

    let app = Router::new()
        .route("/calculate_pp", get(handle_pp_calculation))
        .with_state(beatmap_cache);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8670").await?;
    println!("Server running on http://127.0.0.1:8670");
    axum::serve(listener, app).await?;

    Ok(())
}