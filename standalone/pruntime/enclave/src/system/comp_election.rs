use crate::std::{cmp, vec::Vec};
use log::info;
use rand::{rngs::SmallRng, seq::index::IndexVec, SeedableRng};

use crate::OnlineWorkerSnapshot;

/// Runs an election on `candidates` with `seed` (>64 bits)
pub fn elect(seed: u64, candidates: &OnlineWorkerSnapshot, mid: &Vec<u8>) -> bool {
    let mut rng = SmallRng::seed_from_u64(seed);
    // Collect the weight for each candidate
    let weights = extract_weights(candidates);
    let num_winners = cmp::min(
        candidates.compute_workers as usize,
        weights.len(),
    );
    info!(
        "elect: electing {} winners from {} candidates",
        candidates.compute_workers,
        weights.len()
    );
    // Weighted sample without replacement
    let result = sampling::sample_weighted(&mut rng, weights.len(), |i| weights[i], num_winners)
        .expect("elect: sample_weighted panic");
    // Check if I am a winner by machine id
    let indices = match result {
        IndexVec::U32(indices) => indices,
        _ => panic!("expected `IndexVec::U32`"),
    };
    if indices.len() == 0 {
        panic!("elect: impossible to have 0 elected; qed.");
    }
    let mut hit = false;
    for &i in &indices {
        let worker_info = candidates.worker_state_kv[i as usize].value();
        info!(
            "- winner[{}]: mid={} score={} weight={}",
            i,
            hex::encode(&worker_info.machine_id),
            worker_info.score.as_ref().unwrap().overall_score,
            weights[i as usize],
        );
        if &worker_info.machine_id == mid {
            hit = true;
            info!("elect: hit!");
        }
    }
    hit
}

/// Extracts the weight as an array from `snapshot` in the same order as `worker_state_kv`
fn extract_weights(snapshot: &OnlineWorkerSnapshot) -> Vec<u32> {
    use crate::light_validation::utils::extract_account_id_key_unsafe;
    use crate::std::collections::HashMap;
    let staked: HashMap<_, _> = snapshot
        .stake_received_kv
        .iter()
        .map(|kv| (extract_account_id_key_unsafe(kv.key()), *kv.value()))
        .collect();
    let weights: Vec<_> = snapshot
        .worker_state_kv
        .iter()
        .map(|kv| {
            let account = crate::light_validation::utils::extract_account_id_key_unsafe(kv.key());
            let score = kv.value().score.as_ref().unwrap().overall_score;
            let stake = staked.get(account).cloned().unwrap_or(0);
            weight(score, stake)
        })
        .collect();
    weights
}

/// Calcuates the weight of a worker
///
/// $weight = score + sqrt(stake) * 5$
fn weight(score: u32, staked: u128) -> u32 {
    use fixed::transcendental::sqrt;
    use fixed::types::U64F64 as Fixed;
    const F_1E4: Fixed = Fixed::from_bits(0x2710_0000000000000000); // 1e4 * 1<<64
    const U_1E8: u128 = 100_000_000u128;

    let stake_1e4 = staked / U_1E8;
    let fstaked = Fixed::from_num(stake_1e4) / F_1E4;
    let factor: Fixed = sqrt(fstaked).expect("U128 should never overflow or be negative; qed.");
    let result = score + (factor * 5).to_num::<u32>();
    info!("weight(score={}, staked={}) = {}", score, staked, result);
    result
}

// Backported from rand 0.8
mod sampling {
    use crate::std;
    use crate::std::vec::Vec;
    use anyhow::Result;
    use core::fmt;
    use rand::{distributions::uniform::SampleUniform, seq::index::IndexVec, Rng};

    #[derive(Debug)]
    pub enum WeightedError {
        InvalidWeight,
    }

    impl fmt::Display for WeightedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                WeightedError::InvalidWeight => write!(f, "invalid weight"),
            }
        }
    }

    /// Randomly sample exactly `amount` distinct indices from `0..length`, and
    /// return them in an arbitrary order (there is no guarantee of shuffling or
    /// ordering). The weights are to be provided by the input function `weights`,
    /// which will be called once for each index.
    ///
    /// This method is used internally by the slice sampling methods, but it can
    /// sometimes be useful to have the indices themselves so this is provided as
    /// an alternative.
    ///
    /// This implementation uses `O(length + amount)` space and `O(length)` time
    /// if the "nightly" feature is enabled, or `O(length)` space and
    /// `O(length + amount * log length)` time otherwise.
    ///
    /// Panics if `amount > length`.
    pub fn sample_weighted<R, F, X>(
        rng: &mut R,
        length: usize,
        weight: F,
        amount: usize,
    ) -> Result<IndexVec>
    where
        R: Rng + ?Sized,
        F: Fn(usize) -> X,
        X: Into<f64>,
    {
        if length > (core::u32::MAX as usize) {
            sample_efraimidis_spirakis(rng, length, weight, amount)
        } else {
            assert!(amount <= core::u32::MAX as usize);
            let amount = amount as u32;
            let length = length as u32;
            sample_efraimidis_spirakis(rng, length, weight, amount)
        }
    }

    /// Randomly sample exactly `amount` distinct indices from `0..length`, and
    /// return them in an arbitrary order (there is no guarantee of shuffling or
    /// ordering). The weights are to be provided by the input function `weights`,
    /// which will be called once for each index.
    ///
    /// This implementation uses the algorithm described by Efraimidis and Spirakis
    /// in this paper: https://doi.org/10.1016/j.ipl.2005.11.003
    /// It uses `O(length + amount)` space and `O(length)` time if the
    /// "nightly" feature is enabled, or `O(length)` space and `O(length
    /// + amount * log length)` time otherwise.
    ///
    /// Panics if `amount > length`.
    fn sample_efraimidis_spirakis<R, F, X, N>(
        rng: &mut R,
        length: N,
        weight: F,
        amount: N,
    ) -> Result<IndexVec>
    where
        R: Rng + ?Sized,
        F: Fn(usize) -> X,
        X: Into<f64>,
        N: UInt,
        IndexVec: From<Vec<N>>,
    {
        if amount == N::zero() {
            return Ok(IndexVec::U32(Vec::new()));
        }

        if amount > length {
            panic!("`amount` of samples must be less than or equal to `length`");
        }

        struct Element<N> {
            index: N,
            key: f64,
        }
        impl<N> PartialOrd for Element<N> {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.key.partial_cmp(&other.key)
            }
        }
        impl<N> Ord for Element<N> {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                // partial_cmp will always produce a value,
                // because we check that the weights are not nan
                self.partial_cmp(other).unwrap()
            }
        }
        impl<N> PartialEq for Element<N> {
            fn eq(&self, other: &Self) -> bool {
                self.key == other.key
            }
        }
        impl<N> Eq for Element<N> {}

        #[cfg(feature = "nightly")]
        {
            let mut candidates = Vec::with_capacity(length.as_usize());
            let mut index = N::zero();
            while index < length {
                let weight = weight(index.as_usize()).into();
                if !(weight >= 0.) {
                    return Err(WeightedError::InvalidWeight);
                }

                let key = rng.gen::<f64>().powf(1.0 / weight);
                candidates.push(Element { index, key });

                index += N::one();
            }

            // Partially sort the array to find the `amount` elements with the greatest
            // keys. Do this by using `partition_at_index` to put the elements with
            // the *smallest* keys at the beginning of the list in `O(n)` time, which
            // provides equivalent information about the elements with the *greatest* keys.
            let (_, mid, greater) =
                candidates.partition_at_index(length.as_usize() - amount.as_usize());

            let mut result: Vec<N> = Vec::with_capacity(amount.as_usize());
            result.push(mid.index);
            for element in greater {
                result.push(element.index);
            }
            Ok(IndexVec::from(result))
        }

        #[cfg(not(feature = "nightly"))]
        {
            // #[cfg(all(feature = "alloc", not(feature = "std")))]
            // use crate::alloc::collections::BinaryHeap;
            // #[cfg(feature = "std")]
            use std::collections::BinaryHeap;

            // Partially sort the array such that the `amount` elements with the largest
            // keys are first using a binary max heap.
            let mut candidates = BinaryHeap::with_capacity(length.as_usize());
            let mut index = N::zero();
            while index < length {
                let weight = weight(index.as_usize()).into();
                if !(weight >= 0.) {
                    return Err(anyhow::Error::msg(WeightedError::InvalidWeight));
                }

                let key = rng.gen::<f64>().powf(1.0 / weight);
                candidates.push(Element { index, key });

                index += N::one();
            }

            let mut result: Vec<N> = Vec::with_capacity(amount.as_usize());
            while result.len() < amount.as_usize() {
                result.push(candidates.pop().unwrap().index);
            }
            Ok(IndexVec::from(result))
        }
    }

    trait UInt:
        Copy
        + PartialOrd
        + Ord
        + PartialEq
        + Eq
        + SampleUniform
        + core::hash::Hash
        + core::ops::AddAssign
    {
        fn zero() -> Self;
        fn one() -> Self;
        fn as_usize(self) -> usize;
    }
    impl UInt for u32 {
        #[inline]
        fn zero() -> Self {
            0
        }

        #[inline]
        fn one() -> Self {
            1
        }

        #[inline]
        fn as_usize(self) -> usize {
            self as usize
        }
    }
    impl UInt for usize {
        #[inline]
        fn zero() -> Self {
            0
        }

        #[inline]
        fn one() -> Self {
            1
        }

        #[inline]
        fn as_usize(self) -> usize {
            self
        }
    }
}
