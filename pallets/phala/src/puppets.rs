
pub mod parachain_info {
    pub use pallet::*;

    #[frame_support::pallet]
    pub mod pallet {
        use frame_support::pallet_prelude::*;

        #[pallet::pallet]
        #[pallet::generate_store(pub(super) trait Store)]
        #[pallet::without_storage_info]
        pub struct Pallet<T>(_);

        #[pallet::config]
        pub trait Config: frame_system::Config {}

        #[pallet::call]
        impl<T: Config> Pallet<T> {}

        #[pallet::storage]
        pub(super) type ParachainId<T: Config> = StorageValue<_, ParaId, ValueQuery>;

        #[derive(Encode, Decode, TypeInfo, Default)]
        pub struct ParaId(pub u32);
    }
}

pub mod parachain_system {
    pub use pallet::*;

    #[frame_support::pallet]
    pub mod pallet {
        use frame_support::pallet_prelude::*;

        #[pallet::pallet]
        #[pallet::generate_store(pub(super) trait Store)]
        #[pallet::without_storage_info]
        pub struct Pallet<T>(_);

        #[pallet::config]
        pub trait Config: frame_system::Config {}

        #[pallet::call]
        impl<T: Config> Pallet<T> {}

        #[pallet::storage]
        pub(super) type ValidationData<T: Config> = StorageValue<_, PersistedValidationData>;

        #[derive(Encode, Decode, TypeInfo)]
        pub struct PersistedValidationData {
            relay_parent_number: u32,
        }
    }
}
