//! The TopicsBuilder implementation is copied from ink-env since it is no longer exported by ink-env-3.0.0.
use crate::PinkEnvironment;
use ink_env::{
    hash::{Blake2x256, CryptoHash, HashOutput},
    topics::TopicsBuilderBackend,
    Clear, Environment, Topics,
};

pub fn topics_for(event: impl Topics + scale::Encode) -> Vec<ink_env::Hash> {
    event.topics::<PinkEnvironment, _>(TopicsBuilder::<PinkEnvironment>::new().into())
}

struct TopicsBuilder<E: Environment> {
    topics: Vec<<E as Environment>::Hash>,
}

impl<E> TopicsBuilder<E>
where
    E: Environment,
{
    fn new() -> Self {
        Self { topics: Vec::new() }
    }
}

impl<E> TopicsBuilderBackend<E> for TopicsBuilder<E>
where
    E: Environment,
{
    type Output = Vec<<E as Environment>::Hash>;

    fn expect(&mut self, _expected_topics: usize) {}

    fn push_topic<T>(&mut self, topic_value: &T)
    where
        T: scale::Encode,
    {
        let encoded = topic_value.encode();
        let len_encoded = encoded.len();
        let mut result = <E as Environment>::Hash::clear();
        let len_result = result.as_ref().len();
        if len_encoded <= len_result {
            result.as_mut()[..len_encoded].copy_from_slice(&encoded[..]);
        } else {
            let mut hash_output = <Blake2x256 as HashOutput>::Type::default();
            <Blake2x256 as CryptoHash>::hash(&encoded[..], &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
        }
        self.topics.push(result);
    }

    fn output(self) -> Self::Output {
        self.topics
    }
}
