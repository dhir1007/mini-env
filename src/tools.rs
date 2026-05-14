use crate::types::{ActionRecord, EpisodeState};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn dispatch(
    state: &mut EpisodeState,
    tool: &str,
    args: &HashMap<String, serde_json::Value>,
) -> Result<String> {
    let result = match tool {
        "read_file" => tool_read_file(state, args),
        "write_file" => tool_write_file(state,args),
        "list_dir" => tool_list_dir(state, args),
        "run_tests" => tool_run_tests(state),
        _ => Err(anyhow!("unknown tool: {}. Availabe tools: read_file, write_file, list_dir, run_tests", tool))
    }?;

    state.action_history.push(ActionRecord {
        step: state.steps,
        tool: tool.to_string(),
        args: args.clone(),
        result_summary: result.chars().take(120).collect(),
    });

    Ok(result)
}

fn tool_read_file(state: &EpisodeState, args: &HashMap<String, serde_json::Value>) -> Result<String> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("path is required as an arg"))?;

    let full_path = state.workspace.join(path_str);
    guard_path(&state.workspace, &full_path)?;

    let content = fs::read_to_string(&full_path)
        .map_err(|e| anyhow!("Cannot Read {}: {}", path_str, e))?;

    Ok(format!("=== {} ===\n{}", path_str, content))
}

fn tool_write_file(state: &EpisodeState, args: &HashMap<String, serde_json::Value>) -> Result<String> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("path is required as an arg"))?;

    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("content is required as an arg"))?;

    let full_path = state.workspace.join(path_str);
    guard_path(&state.workspace, &full_path)?;

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&full_path, content)?;
    Ok(format!("Written {} bytes to {}", content.len(), path_str))
}

fn tool_list_dir(state: &EpisodeState, args: &HashMap<String, serde_json::Value>) -> Result<String> {
    let path_str = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("path is required as an arg"))?;

    let full_path = state.workspace.join(path_str);
    guard_path(&state.workspace, &full_path)?;

    let mut entries = Vec::new();

    for entry in fs::read_dir(&full_path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = if entry.file_type()?.is_dir() { "dir" } else { "file"};
        entries.push(format!("[{}] {}", kind, name));
    }

    entries.sort();
    Ok(entries.join("\n"))
}

fn tool_run_tests(state: &EpisodeState) -> Result<String> {
    let output = Command::new("python3")
        .arg("-m")
        .arg("pytest")
        .arg("tests/")
        .arg("-v")
        .arg("--tb=short")
        .current_dir(&state.workspace)
        .output()
        .map_err(|e| anyhow!("Failed to run pytest: {}. Is pytest installed?", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    Ok(combined.chars().take(3000).collect())
}

fn guard_path(workspace: &Path, target: &Path) -> Result<()> {
    let canonical_workspace = workspace.canonicalize()
        .map_err(|e| anyhow!("Cannot canonicalize workspace: {}", e))?;

    let canonical_target = if target.exists() {
        target.canonicalize()
            .map_err(|e| anyhow!("Cannot canonicalize target: {}", e))?
    } else {
        // For files that don't exist yet, check the parent
        let parent = target.parent()
            .ok_or_else(|| anyhow!("No parent directory"))?;
        let canonical_parent = if parent.exists() {
            parent.canonicalize()?
        } else {
            canonical_workspace.clone()
        };
        canonical_parent.join(target.file_name().unwrap_or_default())
    };

    if !canonical_target.starts_with(&canonical_workspace) {
        return Err(anyhow!(
            "Path escape attempt: '{}' is outside the workspace",
            target.display()
        ));
    }
    Ok(())
}