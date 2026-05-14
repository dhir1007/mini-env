use crate::types::*;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn create_episode(task_name: &str, tasks_root: &Path) -> Result<EpisodeState> {
    let task_src = tasks_root.join(task_name);
    if !task_src.exists() {
        return Err(anyhow!("Task '{}' not found in tasks/", task_name));
    }

    //Create isolated workspace under /tmp
    let episode_id = Uuid::new_v4().to_string();
    let workspace = PathBuf::from("/tmp").join(format!("mini-env-{}", episode_id));

    fs::create_dir_all(&workspace)?;

    //copy tasks files to workspace

    copy_dir_recursive(&task_src, &workspace)?;

    Ok(EpisodeState {
        episode_id,
        workspace,
        task_name: task_name.to_string(),
        steps: 0,
        done: false,
        action_history: Vec::new(),
        started_at: Some(std::time::Instant::now()),
    })
} 

pub fn build_observation(state: &EpisodeState) -> String {
    let files = list_workspace_files(&state.workspace);
    format!(
        "Task: Fix the bugs in the codebase so all tes pass. \n\
        Workspace files: {}\n\
        Steps taken: {}\n\
        available tools: read_file, write_file, list_dir, run_tests\n\
        Tip: Start with run_tests to see what's failing.",
        files.join(", "),
        state.steps
    )
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn list_workspace_files(workspace: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files(workspace, workspace, &mut files);
    files
}

fn collect_files(root: &Path, current: &Path, files: &mut Vec<String>) {
    if let Ok(entries) =fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                collect_files(root, &path, files);
            } else if let Ok(rel) = path.strip_prefix(root) {
                files.push(rel.to_string_lossy().to_string());
            }
        }
    }
}

pub fn cleanup_episode(state: &EpisodeState) {
    let _ = fs::remove_dir_all(&state.workspace);
}
