use momento_protos::control_client;
use tonic::Request;

use crate::cache::messages::MomentoRequest;
use crate::{utils, CacheClient, MomentoResult};

/// Request to delete a leaderboard
///
/// # Arguments
///
/// * `cache_name` - The name of the cache containing the leaderboard.
/// * `leaderboard` - The name of the leaderboard.
pub struct DeleteLeaderboardRequest {
    /// The name of the cache containing the leaderboard.
    pub cache_name: String,
    /// The leaderboard to be deleted.
    pub leaderboard: String,
}

impl DeleteLeaderboardRequest {
    /// Constructs a new DeleteCacheRequest.
    pub fn new(cache_name: impl Into<String>, leaderboard: impl Into<String>) -> Self {
        Self {
            cache_name: cache_name.into(),
            leaderboard: leaderboard.into(),
        }
    }
}

impl MomentoRequest for DeleteLeaderboardRequest {
    type Response = ();

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<Self::Response> {
        let cache_name = &self.cache_name;

        utils::is_cache_name_valid(cache_name)?;
        let request = Request::new(control_client::DeleteCacheRequest {
            cache_name: cache_name.to_string(),
        });

        let _ = cache_client.control_client().delete_cache(request).await?;
        Ok(DeleteCacheResponse {})
    }
}

/// The response type for a successful delete cache request
#[derive(Debug, PartialEq, Eq)]
pub struct DeleteCacheResponse {}
