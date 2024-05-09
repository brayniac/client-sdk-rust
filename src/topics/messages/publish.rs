use momento_protos::cache_client::pubsub::TopicValue;

use crate::{
    topics::IntoTopicValue, topics::MomentoRequest, utils::prep_request_with_timeout,
    MomentoResult, TopicClient,
};

/// Publish a value to a topic.
/// The cache is used as a namespace for your topics, and it needs to exist.
/// You don't create topics, you just start using them.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache to use as a namespace for the topic.
/// * `topic` - The name of the topic to publish to.
/// * `value` - The value to publish to the topic.
///
/// # Example
///
/// See [TopicClient] for an example.
pub struct PublishRequest<V: IntoTopicValue> {
    cache_name: String,
    topic: String,
    value: V,
}

impl<V: IntoTopicValue> PublishRequest<V> {
    pub fn new(cache_name: impl Into<String>, topic: impl Into<String>, value: V) -> Self {
        Self {
            cache_name: cache_name.into(),
            topic: topic.into(),
            value,
        }
    }
}

impl<V: IntoTopicValue + std::marker::Send> MomentoRequest for PublishRequest<V> {
    type Response = TopicPublish;

    async fn send(self, topic_client: &TopicClient) -> MomentoResult<TopicPublish> {
        let request = prep_request_with_timeout(
            &self.cache_name.to_string(),
            topic_client.configuration.deadline_millis(),
            momento_protos::cache_client::pubsub::PublishRequest {
                cache_name: self.cache_name,
                topic: self.topic,
                value: Some(TopicValue {
                    kind: Some(self.value.into_topic_value()),
                }),
            },
        )?;

        let _ = topic_client.client.clone().publish(request).await?;
        Ok(TopicPublish {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TopicPublish {}
