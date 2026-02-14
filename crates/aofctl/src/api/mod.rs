//! API modules for aofctl serve command
//!
//! This module provides HTTP API endpoints for the Mission Control UI.

pub mod config;

pub use config::{get_agents_config, get_tools_config, get_config_version};
