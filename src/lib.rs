// Path: src/lib.rs

pub mod db;
pub mod migrations;
pub mod sanitizer;
pub mod security;
pub mod surrealml;
pub mod telemetry;

pub use db::{DatabaseConfig, DatabaseManager};
pub use migrations::{Migration, MigrationError, MigrationManager, MigrationResult};
pub use sanitizer::Sanitizer;
pub use security::SecurityManager;
pub use surrealml::{Dataset, Model, SurrealMLError, SurrealMLStorage};
pub use telemetry::TelemetryManager;
