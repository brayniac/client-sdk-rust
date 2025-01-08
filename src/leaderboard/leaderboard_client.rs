#[derive(Clone, Debug)]
pub struct LeaderboardClient {
    data_clients: Vec<SLbClient<InterceptedService<Channel, HeaderInterceptor>>>,
    control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    configuration: Configuration,
}
