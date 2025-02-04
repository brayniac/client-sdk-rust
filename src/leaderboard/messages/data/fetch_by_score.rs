use super::{fetch::FetchResponse, Order, RankedElement};
use crate::leaderboard::MomentoRequest;
use crate::utils::prep_leaderboard_request_with_timeout;
use crate::{Leaderboard, MomentoResult};

use momento_protos::common::Unbounded;
use momento_protos::leaderboard::score_range::{Max, Min};

use std::ops::Range;

/// Represents a range of scores used to request elements by score.
pub struct ScoreRange {
    min: Option<f64>,
    max: Option<f64>,
}

impl ScoreRange {
    /// Constructs a new `ScoreRange`.
    pub fn new(min: Option<f64>, max: Option<f64>) -> Self {
        Self { min, max }
    }

    /// Constructs a new `ScoreRange` with no bounds.
    pub fn unbounded() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}

impl From<Option<ScoreRange>> for ScoreRange {
    fn from(val: Option<ScoreRange>) -> Self {
        val.unwrap_or_else(|| ScoreRange {
            min: None,
            max: None,
        })
    }
}

impl From<Range<f64>> for ScoreRange {
    fn from(val: std::ops::Range<f64>) -> Self {
        ScoreRange {
            min: Some(val.start),
            max: Some(val.end),
        }
    }
}

impl From<ScoreRange> for momento_protos::leaderboard::ScoreRange {
    fn from(val: ScoreRange) -> Self {
        let min = val
            .min
            // TODO validate score range ie min not pos infinity
            .filter(|&v| v.is_finite())
            .map(Min::MinInclusive)
            .unwrap_or_else(|| Min::UnboundedMin(Unbounded {}));
        let max = val
            .max
            // TODO validate score range ie max not neg infinity
            .filter(|&v| v.is_finite())
            .map(Max::MaxExclusive)
            .unwrap_or_else(|| Max::UnboundedMax(Unbounded {}));

        momento_protos::leaderboard::ScoreRange {
            min: Some(min),
            max: Some(max),
        }
    }
}

/// A request to retrieve ranked elements by score.
pub struct FetchByScoreRequest {
    score_range: ScoreRange,
    offset: Option<u32>,
    count: Option<u32>,
    order: Order,
}

impl FetchByScoreRequest {
    /// Constructs a new `FetchByScoreRequest`.
    pub fn new(score_range: impl Into<ScoreRange>) -> Self {
        Self {
            score_range: score_range.into(),
            offset: None,
            count: None,
            order: Order::Ascending,
        }
    }

    /// Sets the offset of the elements to be fetched.
    pub fn offset(mut self, offset: impl Into<Option<u32>>) -> Self {
        self.offset = offset.into();
        self
    }

    /// Sets the number of elements to be fetched.
    pub fn count(mut self, count: impl Into<Option<u32>>) -> Self {
        self.count = count.into();
        self
    }

    /// Sets the order of the elements to be fetched.
    pub fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

impl MomentoRequest for FetchByScoreRequest {
    type Response = FetchResponse;

    async fn send(self, leaderboard: &Leaderboard) -> MomentoResult<Self::Response> {
        let cache_name = leaderboard.cache_name();
        // TODO validate score range ie min not pos infinity
        let request = prep_leaderboard_request_with_timeout(
            cache_name,
            leaderboard.deadline(),
            momento_protos::leaderboard::GetByScoreRequest {
                cache_name: cache_name.clone(),
                leaderboard: leaderboard.leaderboard_name().clone(),
                score_range: Some(self.score_range.into()),
                offset: self.offset.unwrap_or(0),
                limit_elements: self.count.unwrap_or(8192),
                order: self.order.into_proto() as i32,
            },
        )?;

        let response = leaderboard
            .next_data_client()
            .get_by_score(request)
            .await?
            .into_inner();

        Ok(Self::Response::new(
            response
                .elements
                .iter()
                .map(|v| RankedElement {
                    id: v.id,
                    rank: v.rank,
                    score: v.score,
                })
                .collect(),
        ))
    }
}
