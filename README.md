# mini-env

A minimal RL environment server built in Rust — a toy version of what Polymath Labs
builds at scale for training long-horizon AI agents.

## Architecture

- **Environment server** (`axum`) — stateful, concurrent episodes via `DashMap`
- **Tool system** — agents call `read_file`, `write_file`, `list_dir`, `run_tests`
- **Verifier** — separate binary, read-only, detects test tampering via hash check
- **Episode isolation** — each episode gets its own sandboxed `/tmp` workspace

## The loop

POST /reset  →  seeds isolated workspace, returns episode_id + observation
POST /step   →  agent calls a tool, gets back observation + step reward
POST /verify →  isolated verifier grades the outcome, returns reward breakdown

## Run it

cargo build
cargo run &
python3 agent.py

## What this demonstrates

- Environment/agent separation (the verifier is a separate binary the agent can't reach)
- Reward hacking defense (test file integrity check via hash comparison)
- Concurrent episode support (DashMap, each episode fully isolated)
- Path traversal guard (all tool file access constrained to episode workspace)