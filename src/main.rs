// Path: src/main.rs

use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tracing::{info, warn};

mod anomaly_detection;
mod db;
mod migrations;
mod proto;
mod sanitizer;
mod schema;
mod security;
mod surrealml;
mod telemetry;
mod error;

use crate::db::{DatabaseConfig, DatabaseManager};
use crate::migrations::MigrationManager;
use crate::security::SecurityManager;
use crate::telemetry::TelemetryManager;

use proto::database_service_server::{DatabaseService, DatabaseServiceServer};
use proto::{CreateUserRequest, CreateUserResponse, DeleteUserRequest, DeleteUserResponse, 
           UpdateUserRequest, UpdateUserResponse, GetUserRequest, GetUserResponse};

pub struct DatabaseServiceImpl {
    db: Arc<DatabaseManager>,
    security: Arc<SecurityManager>,
    telemetry: Arc<TelemetryManager>,
}

#[tonic::async_trait]
impl DatabaseService for DatabaseServiceImpl {
    async fn create_user(
        &self,
        request: tonic::Request<CreateUserRequest>,
    ) -> Result<tonic::Response<CreateUserResponse>, tonic::Status> {
        // Create a span for tracing
        let span = self.telemetry.tracer().start("create_user");
        let _guard = span.enter();

        let req = request.into_inner();
        
        // Validate input using SecurityManager
        if !self.security.is_valid_email(&req.email) {
            return Err(tonic::Status::invalid_argument("Invalid email format"));
        }
        if !self.security.is_valid_password(&req.password) {
            return Err(tonic::Status::invalid_argument("Invalid password format"));
        }

        // Hash password
        let hashed_password = self.security.hash_password(&req.password)
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        // Create user
        let user = db::User::new(
            req.email,
            req.name,
            hashed_password,
            req.role,
        );

        match self.db.create_user(&user).await {
            Ok(_) => Ok(tonic::Response::new(CreateUserResponse {
                success: true,
                user_id: user.id.to_string(),
            })),
            Err(e) => {
                warn!("Failed to create user: {}", e);
                Err(tonic::Status::internal("Failed to create user"))
            }
        }
    }

    async fn update_user(
        &self,
        request: tonic::Request<UpdateUserRequest>,
    ) -> Result<tonic::Response<UpdateUserResponse>, tonic::Status> {
        let span = self.telemetry.tracer().start("update_user");
        let _guard = span.enter();

        let req = request.into_inner();
        
        // Find existing user
        let existing_user = self.db.find_user_by_id(&req.user_id).await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .ok_or_else(|| tonic::Status::not_found("User not found"))?;

        // Update user fields
        let updated_user = existing_user.update(
            req.name,
            req.email,
            req.role,
        );

        // Hash password if necessary (not shown here)

        match self.db.update_user(updated_user).await {
            Ok(_) => Ok(tonic::Response::new(UpdateUserResponse {
                success: true,
            })),
            Err(e) => {
                warn!("Failed to update user: {}", e);
                Err(tonic::Status::internal("Failed to update user"))
            }
        }
    }

    async fn delete_user(
        &self,
        request: tonic::Request<DeleteUserRequest>,
    ) -> Result<tonic::Response<DeleteUserResponse>, tonic::Status> {
        let span = self.telemetry.tracer().start("delete_user");
        let _guard = span.enter();

        let req = request.into_inner();
        
        match self.db.delete_user(&req.user_id).await {
            Ok(_) => Ok(tonic::Response::new(DeleteUserResponse {
                success: true,
            })),
            Err(e) => {
                warn!("Failed to delete user: {}", e);
                Err(tonic::Status::internal("Failed to delete user"))
            }
        }
    }

    async fn get_user(
        &self,
        request: tonic::Request<GetUserRequest>,
    ) -> Result<tonic::Response<GetUserResponse>, tonic::Status> {
        let span = self.telemetry.tracer().start("get_user");
        let _guard = span.enter();

        let req = request.into_inner();
        
        match self.db.find_user_by_id(&req.user_id).await {
            Ok(Some(user)) => Ok(tonic::Response::new(GetUserResponse {
                user: Some(user.into()),
            })),
            Ok(None) => Err(tonic::Status::not_found("User not found")),
            Err(e) => {
                warn!("Failed to get user: {}", e);
                Err(tonic::Status::internal("Failed to get user"))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize telemetry
    let telemetry = Arc::new(TelemetryManager::init().await?);
    info!("Telemetry initialized");

    // Initialize security manager
    let security = Arc::new(SecurityManager::new());
    info!("Security manager initialized");

    // Initialize database
    let config = DatabaseConfig {
        url: std::env::var("DB_URL").unwrap_or_else(|_| "ws://localhost:8000".to_string()),
        namespace: std::env::var("DB_NAMESPACE").unwrap_or_else(|_| "test".to_string()),
        database: std::env::var("DB_NAME").unwrap_or_else(|_| "test".to_string()),
        username: std::env::var("DB_USER").unwrap_or_else(|_| "root".to_string()),
        password: std::env::var("DB_PASS").unwrap_or_else(|_| "root".to_string()),
    };

    let db = Arc::new(DatabaseManager::new(config).await?);
    info!("Database connection established");

    // Initialize schema
    schema::init_schema(&db).await?;
    info!("Schema initialized");

    // Initialize migration manager and run migrations
    let migration_manager = MigrationManager::new(db.get_connection().await?, telemetry.clone()).await?;

    // Add migrations
    migration_manager.add_migration(migrations::Migration {
        version: 1,
        name: "Initial schema".to_string(),
        description: "Create initial database schema".to_string(),
        up: r#"
            DEFINE TABLE user SCHEMAFULL;
            DEFINE FIELD id ON user TYPE string;
            DEFINE FIELD email ON user TYPE string;
            DEFINE FIELD name ON user TYPE string;
            DEFINE FIELD password_hash ON user TYPE string;
            DEFINE FIELD role ON user TYPE string;
            DEFINE FIELD created_at ON user TYPE datetime;
            DEFINE FIELD updated_at ON user TYPE datetime;
            DEFINE INDEX user_email ON user FIELDS email UNIQUE;
        "#.to_string(),
        down: "REMOVE TABLE user;".to_string(),
        applied_at: None,
    });

    migration_manager.add_migration(migrations::Migration {
        version: 2,
        name: "Add user roles".to_string(),
        description: "Add role-based access control".to_string(),
        up: r#"
            DEFINE FIELD permissions ON user TYPE array;
            DEFINE FIELD last_login ON user TYPE datetime;
        "#.to_string(),
        down: r#"
            REMOVE FIELD permissions ON user;
            REMOVE FIELD last_login ON user;
        "#.to_string(),
        applied_at: None,
    });

    migration_manager.run_pending_migrations().await?;
    info!("Migrations completed");

    // Create service implementation
    let service = DatabaseServiceImpl {
        db: db.clone(),
        security,
        telemetry: telemetry.clone(),
    };

    // Start gRPC server
    let addr = "[::1]:50051".parse()?;
    let listener = TcpListener::bind(&addr).await?;
    info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(DatabaseServiceServer::new(service))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
        .await?;

    Ok(())
}
