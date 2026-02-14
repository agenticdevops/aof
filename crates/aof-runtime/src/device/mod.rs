//! Device pairing and mTLS authentication infrastructure.
//!
//! This module provides:
//! - Private CA for issuing client certificates
//! - Device registry with approval workflows
//! - mTLS configuration for secure client authentication
//! - Certificate lifecycle management

pub mod ca;
pub mod certificate;
pub mod registry;
pub mod mtls;

pub use ca::PrivateCA;
pub use certificate::CertificateManager;
pub use registry::DeviceRegistry;
pub use mtls::MtlsConfig;
