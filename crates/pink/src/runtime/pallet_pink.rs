pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use pallet_contracts::AddressGenerator;
    use phala_crypto::sr25519::Sr25519SecretKey;
    use scale::{Decode, Encode};
    use scale_info::TypeInfo;
    use sp_core::crypto::UncheckedFrom;
    use sp_runtime::traits::Hash as _;

    type CodeHash<T> = <T as frame_system::Config>::Hash;

    #[derive(Clone, Eq, PartialEq, Encode, Decode, TypeInfo)]
    pub struct WasmCode<AccountId> {
        pub owner: AccountId,
        pub code: Vec<u8>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::storage]
    pub(crate) type ClusterId<T: Config> = StorageValue<_, Vec<u8>, ValueQuery>;

    /// The seed used to derive custom keys in `ink!` contract.
    ///
    /// All contracts in a cluster shares the same seed. When deriving a key from the seed, the
    /// contract address is appended to the seed to avoid collisions.
    #[pallet::storage]
    #[pallet::getter(fn key_seed)]
    pub(crate) type KeySeed<T: Config> = StorageValue<_, Sr25519SecretKey>;

    /// Uploaded sidevm codes
    #[pallet::storage]
    #[pallet::getter(fn sidevm_codes)]
    pub(crate) type SidevmCodes<T: Config> =
        StorageMap<_, Twox64Concat, T::Hash, WasmCode<T::AccountId>>;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T: Config> AddressGenerator<T> for Pallet<T>
    where
        T::AccountId: UncheckedFrom<T::Hash> + AsRef<[u8]>,
    {
        fn generate_address(
            deploying_address: &T::AccountId,
            code_hash: &CodeHash<T>,
            salt: &[u8],
        ) -> T::AccountId {
            let cluster_id = <ClusterId<T>>::get();
            let buf = phala_types::contract::contract_id_preimage(
                deploying_address.as_ref(),
                code_hash.as_ref(),
                &cluster_id,
                salt,
            );
            UncheckedFrom::unchecked_from(<T as frame_system::Config>::Hashing::hash(&buf))
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn set_cluster_id(cluster_id: &[u8]) {
            <ClusterId<T>>::put(cluster_id.to_vec());
        }

        pub fn set_key_seed(seed: Sr25519SecretKey) {
            <KeySeed<T>>::put(seed);
        }

        pub fn put_sidevm_code(owner: T::AccountId, code: Vec<u8>) -> T::Hash {
            let hash = T::Hashing::hash(&code);
            <SidevmCodes<T>>::insert(hash, WasmCode { owner, code });
            hash
        }
    }
}
