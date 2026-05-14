use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeState {
    pub episode_id: String,
    pub workspace: PathBuf,
    pub task_name: String,
    pub steps: u32,
    pub done: bool,
    pub action_history: Vec<ActionRecord>,
    #[serde(skip)]
    pub started_at: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub step: u32,
    pub tool: String,
    pub args: HashMap<String, serde_json::Value>,
    pub result_summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetResponse {
    pub episode_id: String,
    pub observation: String,
    pub task_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepRequest {
    pub episode_id: String,
    pub tool: String,
    pub args: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepResponse {
    pub observation: String,
    pub reward: f32,
    pub done: bool,
    pub info: StepInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StepInfo {
    pub step: u32,
    pub tool_used: String,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub episode_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub reward: f32,
    pub correctness: f64,
    pub test_integrity: bool,
    pub tests_passed: u32,
    pub tests_total: u32,
    pub breakdown: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetRequest {
    pub task: Option<String>,
}