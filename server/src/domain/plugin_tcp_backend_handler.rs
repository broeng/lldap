use std::collections::HashSet;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use lldap_domain::types::UserId;
use lldap_domain_model::error::Result;
use lldap_plugin_backend_handler::handler::PluginBackendHandler;

use crate::tcp_backend_handler::TcpBackendHandler;

// This implementation is in the server crate, because `TcpBackendHandler` is
// defined in this crate. There are no plugin events for these operations, thus
// they go directly to the backend.
#[async_trait]
impl TcpBackendHandler for PluginBackendHandler {
    async fn get_jwt_blacklist(&self) -> anyhow::Result<HashSet<u64>> {
        self.backend_handler.get_jwt_blacklist().await
    }
    async fn create_refresh_token(&self, user: &UserId) -> Result<(String, chrono::Duration)> {
        self.backend_handler.create_refresh_token(user).await
    }
    async fn register_jwt(
        &self,
        user: &UserId,
        jwt_hash: u64,
        expiry_date: NaiveDateTime,
    ) -> Result<()> {
        self.backend_handler
            .register_jwt(user, jwt_hash, expiry_date)
            .await
    }
    async fn check_token(&self, refresh_token_hash: u64, user: &UserId) -> Result<bool> {
        self.backend_handler
            .check_token(refresh_token_hash, user)
            .await
    }
    async fn blacklist_jwts(&self, user: &UserId) -> Result<HashSet<u64>> {
        self.backend_handler.blacklist_jwts(user).await
    }
    async fn delete_refresh_token(&self, refresh_token_hash: u64) -> Result<()> {
        self.backend_handler
            .delete_refresh_token(refresh_token_hash)
            .await
    }
    async fn start_password_reset(&self, user: &UserId) -> Result<Option<String>> {
        self.backend_handler.start_password_reset(user).await
    }
    async fn get_user_id_for_password_reset_token(&self, token: &str) -> Result<UserId> {
        self.backend_handler
            .get_user_id_for_password_reset_token(token)
            .await
    }
    async fn delete_password_reset_token(&self, token: &str) -> Result<()> {
        self.backend_handler
            .delete_password_reset_token(token)
            .await
    }
}
