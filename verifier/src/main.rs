use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Serialize, Deserialize)]
struct VerifyResponse {
    reward: f32,
    correctness: f32,
    test_integrity: bool,
    tests_passed: u32,
    tests_total: u32,
    breakdown: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: verifier <workspace_path> <original_tests_path>");
        std::process::exit(1);
    }

    let workspace = PathBuf::from(&args[1]);
    let original_tests = PathBuf::from(&args[2]);

    // 1. Check test file integrity (reward hacking defense)
    let test_integrity = check_test_integrity(&workspace, &original_tests);

    // 2. Run tests from a COPY of the workspace to prevent any mid-verify mutation
    let (passed, total) = run_tests_isolated(&workspace);

    // 3. Compute scores
    let correctness = if total == 0 { 0.0 } else { passed as f32 / total as f32 };

    // Integrity penalty: if tests were mutated, cap correctness at 0
    let effective_correctness = if test_integrity { correctness } else { 0.0 };

    let reward = effective_correctness * 0.9 + if test_integrity { 0.1 } else { 0.0 };

    let breakdown = format!(
        "Tests: {}/{} passed | Integrity: {} | Correctness: {:.0}% | Reward: {:.2}",
        passed,
        total,
        if test_integrity { "OK" } else { "TAMPERED - score zeroed" },
        effective_correctness * 100.0,
        reward
    );

    let response = VerifyResponse {
        reward,
        correctness: effective_correctness,
        test_integrity,
        tests_passed: passed,
        tests_total: total,
        breakdown,
    };

    println!("{}", serde_json::to_string(&response).unwrap());
}

fn check_test_integrity(workspace: &Path, original_tests: &Path) -> bool {
    let workspace_tests = workspace.join("tests");
    let originals = hash_dir(original_tests);
    let current = hash_dir(&workspace_tests);

    for (filename, original_hash) in &originals {
        match current.get(filename) {
            None => return false, // test file deleted
            Some(h) if h != original_hash => return false, // test file modified
            _ => {}
        }
    }
    true
}

fn hash_dir(dir: &Path) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = fs::read(&path) {
                    let hash = djb2(&content);
                    let name = entry.file_name().to_string_lossy().to_string();
                    map.insert(name, hash);
                }
            }
        }
    }
    map
}

fn djb2(data: &[u8]) -> u64 {
    let mut hash: u64 = 5381;
    for &byte in data {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

fn run_tests_isolated(workspace: &Path) -> (u32, u32) {
    // Run pytest and parse the summary line
    let output = Command::new("python3")
        .arg("-m")
        .arg("pytest")
        .arg("tests/")
        .arg("-v")
        .arg("--tb=no")
        .arg("-q")
        .current_dir(workspace)
        .output();

    match output {
        Err(_) => (0, 0),
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_pytest_summary(&stdout)
        }
    }
}

fn parse_pytest_summary(output: &str) -> (u32, u32) {
    let mut passed = 0u32;
    let mut failed = 0u32;

    for line in output.lines() {
        let line = line.trim();
        // Match lines like "4 passed in 0.01s" or "3 passed, 1 failed in 0.01s"
        if !line.contains("passed") && !line.contains("failed") && !line.contains("error") {
            continue;
        }
        for part in line.split(',') {
            let part = part.trim();
            // strip everything after "in X.XXs"
            let part = if let Some(idx) = part.find(" in ") {
                &part[..idx]
            } else {
                part
            };
            let part = part.trim();
            if let Some(n_str) = part.split_whitespace().next() {
                if let Ok(n) = n_str.parse::<u32>() {
                    if part.contains("passed") { passed = n; }
                    if part.contains("failed") { failed += n; }
                    if part.contains("error") { failed += n; }
                }
            }
        }
    }

    (passed, passed + failed)
}