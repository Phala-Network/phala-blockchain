use crate::std::{cmp, vec::Vec};
use rand::{
    SeedableRng,
    rngs::SmallRng,
    seq::index::IndexVec,
};

/// Runs an election on `candidates` with `seed` (>64 bits)
pub fn elect(seed: u64, candidates: &crate::OnlineWorkerSnapshot, mid: &Vec<u8>) -> bool {
    let mut rng = SmallRng::seed_from_u64(seed);
    // Collect the weight for each candidate
    let weights: Vec<_> = candidates.worker_state_kv
        .iter()
        .map(|kv| kv.value().score.as_ref().unwrap().overall_score)
        .collect();
    let num_winners = cmp::min(*candidates.compute_workers_kv.value() as usize, weights.len());
    println!(
        "elect: electing {} winners from {} candidates",
        candidates.compute_workers_kv.value(),
        weights.len());
    // Weighted sample without replacement
    let result = sampling::sample_weighted(
        &mut rng, weights.len(),
        |i| weights[i],
        num_winners
    ).expect("elect: sample_weighted panic");
    // Check if I am a winner by machine id
    let indices = match result {
        IndexVec::U32(indices) => indices,
        _ => panic!("expected `IndexVec::U32`"),
    };
    for &i in &indices {
        let worker_info = candidates.worker_state_kv[i as usize].value();
        println!(
            "- winner[{}]: mid={} score={}",
            i,
            crate::hex::encode_hex_compact(&worker_info.machine_id),
            worker_info.score.as_ref().unwrap().overall_score,
        );
        if &worker_info.machine_id == mid {
            println!("elect: hit!");
            return true
        }
    }
    if indices.len() == 0 {
        panic!("elect: impossible to have 0 elected; qed.");
    }
    false
}

// Backported from rand 0.8
mod sampling {
    use crate::std;
    use crate::std::vec::Vec;
    use rand::{
        Rng,
        seq::index::IndexVec,
        distributions::uniform::SampleUniform,
    };

    #[derive(Debug)]
    pub enum WeightedError {
        InvalidWeight,
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
        rng: &mut R, length: usize, weight: F, amount: usize,
    ) -> Result<IndexVec, WeightedError>
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
        rng: &mut R, length: N, weight: F, amount: N,
    ) -> Result<IndexVec, WeightedError>
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
            let (_, mid, greater)
                = candidates.partition_at_index(length.as_usize() - amount.as_usize());

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
                    return Err(WeightedError::InvalidWeight);
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

    trait UInt: Copy + PartialOrd + Ord + PartialEq + Eq + SampleUniform
        + core::hash::Hash + core::ops::AddAssign {
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
