//! Heimdall core: parsing and domain logic for Claude Code data.
//!
//! This crate must never depend on `tauri` (ADR 0002). Every function takes the Claude home
//! and any other root as a parameter so tests run on fixture trees on any OS.

// Diagnostics travel as AppError values, never as prints (ADR 0005, port-plan P-17).
#![deny(clippy::print_stdout, clippy::print_stderr)]

pub mod error;
pub mod classify;
pub mod config;
pub mod data;
pub mod diff;
pub mod export;
pub mod jsonl;
pub mod mcp_status;
pub mod models;
pub mod os_integration;
pub mod paths;
pub mod projects;
pub mod search;
pub mod transcript;

pub use error::{AppError, Loaded, Result, WriteOutcome};
