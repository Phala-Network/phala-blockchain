#![cfg_attr(not(feature = "std"), no_std)]

use pink_extension as pink;

#[pink::contract(env=PinkEnvironment)]
mod unittests {
    use super::pink;
    use pink::PinkEnvironment;

    #[ink(storage)]
    pub struct Unittests {}

    impl Unittests {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn test(&self) {}
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn getrandom_works() {
            pink_extension_runtime::mock_ext::mock_all_ext();

            let bytes = pink::ext().getrandom(3);
            assert_eq!(bytes.len(), 3);
            assert!(bytes != [0; 3]);
        }
    }
}
