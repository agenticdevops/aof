//! Error types for the AOF Skills platform.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur in the skills platform
#[derive(Error, Debug)]
pub enum SkillError {
    /// Failed to read a skill file
    #[error("Failed to read skill file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    /// Failed to parse frontmatter
    #[error("Failed to parse frontmatter in '{path}': {message}")]
    FrontmatterError {
        path: PathBuf,
        message: String,
    },

    /// Invalid skill structure
    #[error("Invalid skill structure in '{path}': {message}")]
    InvalidSkill {
        path: PathBuf,
        message: String,
    },

    /// Skill not found
    #[error("Skill not found: {name}")]
    NotFound {
        name: String,
    },

    /// Requirements not met
    #[error("Skill '{name}' requirements not met: {details}")]
    RequirementsNotMet {
        name: String,
        details: String,
    },

    /// Registry error
    #[error("Registry error: {message}")]
    RegistryError {
        message: String,
    },

    /// File watcher error
    #[error("File watcher error: {message}")]
    WatcherError {
        message: String,
    },

    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Glob pattern error
    #[error("Glob pattern error: {0}")]
    GlobError(#[from] glob::PatternError),
}

impl SkillError {
    /// Create a read error
    pub fn read_error(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::ReadError {
            path: path.into(),
            source,
        }
    }

    /// Create a frontmatter error
    pub fn frontmatter_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::FrontmatterError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create an invalid skill error
    pub fn invalid_skill(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::InvalidSkill {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(name: impl Into<String>) -> Self {
        Self::NotFound { name: name.into() }
    }

    /// Create a requirements not met error
    pub fn requirements_not_met(name: impl Into<String>, details: impl Into<String>) -> Self {
        Self::RequirementsNotMet {
            name: name.into(),
            details: details.into(),
        }
    }

    /// Create a registry error
    pub fn registry_error(message: impl Into<String>) -> Self {
        Self::RegistryError {
            message: message.into(),
        }
    }

    /// Create a watcher error
    pub fn watcher_error(message: impl Into<String>) -> Self {
        Self::WatcherError {
            message: message.into(),
        }
    }
}
