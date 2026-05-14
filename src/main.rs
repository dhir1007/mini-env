mod env;
mod tools;
mod types;

use axum::{extract::State, http::StatusCode, response::Json, routing::post, Router};

use dashMap::DashMap;
use std::path::PathBuf;
use std::sync::Arc;
use types::*;

#[derive(Clone)]
struct AppState {
    episodes: Arc<DashMap<String, EpisodeState>>,
    tasks_root: PathBuf,
    verifier_bin: PathBuf,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let tasks_root = PathBuf::from("tasks");
    let verifier_bin = PathBuf::from("target/debug/verifier");

    if !tasks_root.exists() {
        eprintln!("Error: tasks/ directory not found. Run from mini-env root.");
        std::process::exit(1);
    }

    let state = AppState {
        episodes: Arc::new(DashMap::new()),
        tasks_root,
        verifier_bin,
    };

    let app = Router::new()
        .route("/reset", post(handle_reset))
        .route("/step", post(handle_step))
        .route("/verifiy", post(handle_verify))
        .with_state(state);

    let addr = "0.0.0.0:8080";
    tracing::info!("mini-env listening on {}", addr);
    println!("mini-env running at http://{}", addr);
    println!("Endpoints: /reset, /step, /verifiy");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handle_reset(
    State(state): State<AppState>,
    Json(req): Json<ResetRequest>,
) -> Result<Json<ResetResponse>, (StatusCode, Json<ErrorResponse>)> {
    let task = req.task.as_deref().unwrap_or("buggy");

    let episode = env::create_episode(task, &state.tasks_root).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let obs = env::build_observation(&episode);
    let episode_id = episode.episode_id.clone();

    state.episodes.insert(episode_id.clone(), episode);

    tracing::info!("Episode {} created for task {}", episode_id, task);

    Ok(Json(ResetResponse {
        episode_id,
        observation: obs,
        task_description: "Fix the bug in solution.py so all tests pass.".to_string(),
    }))
}

async fn handle_step(
    State(state): State<AppState>,
    Json(req): Json<StepRequest>,
) -> Result<Json<StepResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut entry = state.episodes.get_mut(&req.episode_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Episode {} not found", req.episode_id),
            }),
        )
    })?;

    if entry.done {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Episode already done".to_string(),
            }),
        ));
    }

    entry.steps += 1;
    let step = entry.steps;

    let obs = tools::dispatch(&mut entry, &req.tool, &req.args)
        .unwrap_or_else(|e| format!("Tool Error: {}", e));

    // small step penalty to encourage efficiency
    let step_reward: f32 = -0.01;

    //mark done after 30 steps

    let done = step >= 30;

    if done {
        entry.done = true;
    }
    let success = !obs.starts_with("Tool Error");

    Ok(Json(StepResponse {
        observation: obs,
        reward: step_reward,
        done,
        info: StepInfo {
            step,
            tool_used: req.tool.clone(),
            success,
        },
    }))
}

async fn handle_verify(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let entry = state.episodes.get(&req.episode_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Episode '{}' not found", req.episode_id),
            }),
        )
    })?;

    // Run verifier as a separate process — it cannot be influenced by the agent
    let output = std::process::Command::new(&state.verifier_bin)
        .arg(&entry.workspace)
        .arg("tasks/buggy/tests") // original tests, read-only reference
        .output()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!(
                        "Verifier failed to run: {}. Did you build it? cargo build --bin verifier",
                        e
                    ),
                }),
            )
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str::<VerifyResponse>(&stdout)
        .map(Json)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Verifier output parse error: {}. Output was: {}", e, stdout),
                }),
            )
        })
}
