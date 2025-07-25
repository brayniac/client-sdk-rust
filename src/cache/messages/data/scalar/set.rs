use momento_protos::cache_client::ECacheResult;

use crate::cache::messages::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::{CacheClient, MomentoError};
use crate::{IntoBytes, MomentoResult};
use std::time::Duration;

/// Request to set a value in a cache.
///
/// # Arguments
///
/// * `cache_name` - name of the cache
/// * `key` - key of the item whose value we are setting
/// * `value` - data to stored in the cache item
///
/// # Optional Arguments
///
/// * `ttl` - The time-to-live for the item. If not provided, the client's default time-to-live is used.
///
/// # Example
/// Assumes that a CacheClient named `cache_client` has been created and is available.
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # use momento_test_util::create_doctest_cache_client;
/// # tokio_test::block_on(async {
/// use std::time::Duration;
/// use momento::cache::{SetResponse, SetRequest};
/// use momento::MomentoErrorCode;
/// # let (cache_client, cache_name) = create_doctest_cache_client();
///
/// let set_request = SetRequest::new(
///     &cache_name,
///     "key",
///     "value1"
/// ).ttl(Duration::from_secs(60));
///
/// match cache_client.send_request(set_request).await {
///     Ok(_) => println!("SetResponse successful"),
///     Err(e) => if let MomentoErrorCode::CacheNotFoundError = e.error_code {
///         println!("Cache not found: {}", &cache_name);
///     } else {
///         eprintln!("Error setting value in cache {}: {}", &cache_name, e);
///     }
/// }
/// # Ok(())
/// # })
/// # }
/// ```
pub struct SetRequest<K: IntoBytes, V: IntoBytes> {
    cache_name: String,
    key: K,
    value: V,
    ttl: Option<Duration>,
}

impl<K: IntoBytes, V: IntoBytes> SetRequest<K, V> {
    /// Construct a new SetRequest.
    pub fn new(cache_name: impl Into<String>, key: K, value: V) -> Self {
        let ttl = None;
        Self {
            cache_name: cache_name.into(),
            key,
            value,
            ttl,
        }
    }

    /// Set the time-to-live for the item.
    pub fn ttl(mut self, ttl: impl Into<Option<Duration>>) -> Self {
        self.ttl = ttl.into();
        self
    }
}

impl<K: IntoBytes, V: IntoBytes> MomentoRequest for SetRequest<K, V> {
    type Response = SetResponse;

    async fn send(self, cache_client: &CacheClient) -> MomentoResult<SetResponse> {
        let request = prep_request_with_timeout(
            &self.cache_name,
            cache_client.deadline_millis(),
            momento_protos::cache_client::SetRequest {
                cache_key: self.key.into_bytes(),
                cache_body: self.value.into_bytes(),
                ttl_milliseconds: cache_client.expand_ttl_ms(self.ttl)?,
            },
        )?;

        let response = cache_client
            .next_data_client()
            .set(request)
            .await?
            .into_inner();
        match response.result() {
            ECacheResult::Ok => Ok(SetResponse {}),
            _ => Err(MomentoError::unknown_error(
                "Set",
                Some(format!("{response:#?}")),
            )),
        }
    }
}

/// The response type for a successful set request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SetResponse {}
