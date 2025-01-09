use std::time::Duration;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicUsize;
use crate::leaderboard::MomentoRequest;
use crate::leaderboard::messages::data::get_rank::{GetRankRequest, GetRankResponse, IntoIds, Order};
use crate::leaderboard::messages::data::upsert_elements::UpsertElementsRequest;
use momento_protos::common::Empty;
use crate::MomentoResult;
use crate::leaderboard::messages::data::upsert_elements::IntoElements;
use crate::leaderboard::leaderboard_client_builder::NeedsConfiguration;
use crate::leaderboard::leaderboard_client_builder::LeaderboardClientBuilder;
use tonic::transport::Channel;
use crate::grpc::header_interceptor::HeaderInterceptor;
use momento_protos::control_client::scs_control_client::ScsControlClient;
use crate::leaderboard::Configuration;
use momento_protos::leaderboard::leaderboard_client::LeaderboardClient as SLbClient;
use tonic::codegen::InterceptedService;

static NEXT_DATA_CLIENT_INDEX: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug)]
pub struct LeaderboardClient {
    data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>>,
    #[allow(dead_code)]
    control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    configuration: Configuration,
}

impl LeaderboardClient {
    pub fn builder() -> LeaderboardClientBuilder<NeedsConfiguration> {
        LeaderboardClientBuilder(NeedsConfiguration{ })
    }

    // pub async fn sorted_set_put_elements<V: IntoBytes>(
    //     &self,
    //     cache_name: impl Into<String>,
    //     sorted_set_name: impl IntoBytes,
    //     elements: impl IntoElements<V>,
    // ) -> MomentoResult<SortedSetPutElementsResponse> {
    //     let request = SortedSetPutElementsRequest::new(cache_name, sorted_set_name, elements);
    //     request.send(self).await
    // }

    pub async fn get_rank<T: IntoIds>(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        ids: impl IntoIds,
        order: Order,
    ) -> MomentoResult<GetRankResponse> {
        let request = GetRankRequest::new(cache_name, leaderboard, ids, order);
        request.send(self).await
    }


    pub async fn upsert_elements<E: IntoElements>(
        &self,
        cache_name: impl Into<String>,
        leaderboard: impl Into<String>,
        elements: impl IntoElements,
    ) -> MomentoResult<Empty> {
        let request = UpsertElementsRequest::new(cache_name, leaderboard, elements);
        request.send(self).await
    }

    /* helper fns */
    pub(crate) fn new(
        data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>>,
        control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
        configuration: Configuration,
    ) -> Self {
        Self {
            data_clients,
            control_client,
            configuration,
        }
    }

    pub(crate) fn deadline_millis(&self) -> Duration {
        self.configuration.deadline_millis()
    }

    #[allow(dead_code)]
    pub(crate) fn control_client(
        &self,
    ) -> ScsControlClient<InterceptedService<Channel, HeaderInterceptor>> {
        self.control_client.clone()
    }

    pub(crate) fn next_data_client(
        &self,
    ) -> SLbClient<InterceptedService<Channel, HeaderInterceptor>> {
        let next_index =
            NEXT_DATA_CLIENT_INDEX.fetch_add(1, Ordering::Relaxed) % self.data_clients.len();
        self.data_clients[next_index].clone()
    }
}
