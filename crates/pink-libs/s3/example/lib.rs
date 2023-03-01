#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod example {

    #[ink(storage)]
    pub struct Example {}

    impl Example {
        #[ink(constructor)]
        pub fn default() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn it_works(&self) {
            use pink_s3 as s3;

            // I don't care to expose them.
            let endpoint = "s3.kvin.wang:8443";
            let region = "garage";
            let access_key = env!("S3_ACCESS_KEY");
            let secret_key = env!("S3_SECRET_KEY");

            let s3 = s3::S3::new(endpoint, region, access_key, secret_key)
                .unwrap()
                .virtual_host_mode(); // virtual host mode is required for newly created AWS S3 buckets.

            let bucket = "fat-1";
            let object_key = "path/to/foo";
            let value = b"bar";

            s3.put(bucket, object_key, value).unwrap();

            let head = s3.head(bucket, object_key).unwrap();
            assert_eq!(head.content_length, value.len() as u64);

            let v = s3.get(bucket, object_key).unwrap();
            assert_eq!(v, value);

            s3.delete(bucket, object_key).unwrap();
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        use super::*;

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            pink_extension_runtime::mock_ext::mock_all_ext();

            let example = Example::default();
            example.it_works();
        }
    }
}
