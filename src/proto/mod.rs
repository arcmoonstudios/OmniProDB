use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserResponse {
    pub success: bool,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub user_id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserResponse {
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserResponse {
    pub user: Option<User>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

pub mod database_service_server {
    use super::*;
    use async_trait::async_trait;
    use tonic::{Request, Response, Status};

    pub trait DatabaseService: Send + Sync + 'static {
        async fn create_user(
            &self,
            request: Request<CreateUserRequest>,
        ) -> Result<Response<CreateUserResponse>, Status>;

        async fn update_user(
            &self,
            request: Request<UpdateUserRequest>,
        ) -> Result<Response<UpdateUserResponse>, Status>;

        async fn delete_user(
            &self,
            request: Request<DeleteUserRequest>,
        ) -> Result<Response<DeleteUserResponse>, Status>;

        async fn get_user(
            &self,
            request: Request<GetUserRequest>,
        ) -> Result<Response<GetUserResponse>, Status>;
    }

    pub struct DatabaseServiceServer<T: DatabaseService>(pub T);
} 