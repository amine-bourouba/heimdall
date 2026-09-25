//! Configuration writers. Every write goes through json_doc and atomic (ADR 0009).

pub mod atomic;
pub mod files;
pub mod json_doc;
pub mod mcp;
pub mod settings;
