use crate::LeaderboardClient;
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_request_with_timeout;
use crate::MomentoResult;

#[repr(i32)]
pub enum Order {
    Ascending = 0,
    Descending = 1,
}

/// This trait defines an interface for converting a type into a vector of [SortedSetElement].
pub trait IntoIds: Send {
    /// Converts the type into a vector of [SortedSetElement].
    fn into_ids(self) -> Vec<u32>;
}

#[cfg(not(doctest))]
pub fn map_and_collect_elements<'a, I>(iter: I) -> Vec<u32>
where
    I: Iterator<Item = &'a u32>,
{
    iter.copied().collect()
}

impl IntoIds for Vec<u32> {
    fn into_ids(self) -> Vec<u32> {
        self
    }
}

impl IntoIds for &[u32] {
    fn into_ids(self) -> Vec<u32> {
        map_and_collect_elements(self.iter())
    }
}

/// Represents an element in a leaderboard.
#[derive(Debug, PartialEq, Clone)]
pub struct RankedElement {
    /// The id of the element.
    pub id: u32,
    // The rank of the element within the leaderboard.
    pub rank: u32,
    /// The score associated with the element.
    pub score: f64,
}

pub struct GetRankRequest {
    cache_name: String,
    leaderboard: String,
    ids: Vec<u32>,
    order: Order,
}

pub struct GetRankResponse {
    elements: Vec<RankedElement>,
}

impl GetRankRequest {
    /// Constructs a new SortedSetPutElementsRequest.
    pub fn new(cache_name: String, leaderboard: String, ids: Vec<u32>, order: Order) -> Self {
        Self {
            cache_name,
            leaderboard,
            ids,
            order,
        }
    }
}

impl GetRankResponse {
    pub fn elements(&self) -> &[RankedElement] {
        &self.elements
    }
}

// impl Iterator for GetRankResponseIter {
//     type Item = RankedElement;

//     fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {

//     }
// }

impl MomentoRequest
    for GetRankRequest
{
    type Response = GetRankResponse;

    async fn send(self, leaderboard_client: &LeaderboardClient) -> MomentoResult<Self::Response> {
        let ids = self.ids.into_ids();
        let cache_name = self.cache_name.clone();
        let request = prep_request_with_timeout(
            &self.cache_name,
            leaderboard_client.deadline_millis(),
            momento_protos::leaderboard::GetRankRequest {
                cache_name,
                leaderboard: self.leaderboard,
                ids,
                order: self.order as i32,
            },
        )?;

        let response = leaderboard_client
            .next_data_client()
            .get_rank(request)
            .await?
            .into_inner();

        Ok(GetRankResponse {
            elements: response.elements.iter().map(|v| RankedElement { id: v.id, rank: v.rank, score: v.score}).collect()
        })
    }
}
