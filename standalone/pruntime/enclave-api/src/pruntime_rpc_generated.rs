/// Basic information about a Phactory instance.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PhactoryInfo {
    /// Whether the init_runtime has been called successfully.
    #[prost(bool, tag = "1")]
    pub initialized: bool,
    /// Whether the worker has been registered on-chain.
    #[prost(bool, tag = "2")]
    pub registered: bool,
    /// The Gatekeeper role of the worker.
    #[prost(enumeration = "phactory_info::GatekeeperRole", tag = "3")]
    pub gatekeeper_role: i32,
    /// Genesis block header hash passed in by init_runtime.
    #[prost(string, optional, tag = "4")]
    pub genesis_block_hash: ::core::option::Option<::prost::alloc::string::String>,
    /// Public key of the worker.
    #[prost(string, optional, tag = "5")]
    pub public_key: ::core::option::Option<::prost::alloc::string::String>,
    /// ECDH public key of the worker.
    #[prost(string, optional, tag = "6")]
    pub ecdh_public_key: ::core::option::Option<::prost::alloc::string::String>,
    /// The relaychain/solochain header number synchronized to.
    #[prost(uint32, tag = "7")]
    pub headernum: u32,
    /// The parachain header number synchronized to. (parachain mode only)
    #[prost(uint32, tag = "8")]
    pub para_headernum: u32,
    /// The changes block number synchronized to.
    #[prost(uint32, tag = "9")]
    pub blocknum: u32,
    /// Current chain storage's state root.
    #[prost(string, tag = "10")]
    pub state_root: ::prost::alloc::string::String,
    /// Whether the worker is running in dev mode.
    #[prost(bool, tag = "11")]
    pub dev_mode: bool,
    /// The number of mq messages in the egress queue.
    #[prost(uint64, tag = "12")]
    pub pending_messages: u64,
    /// Local estimated benchmark score.
    #[prost(uint64, tag = "13")]
    pub score: u64,
}
/// Nested message and enum types in `PhactoryInfo`.
pub mod phactory_info {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum GatekeeperRole {
        None = 0,
        Dummy = 1,
        Active = 2,
    }
}
#[doc = r" Generated client implementations."]
pub mod phactory_api_client {
    #[doc = " The greeting service definition."]
    #[derive(Debug)]
    pub struct PhactoryApiClient<Client> {
        client: Client,
    }
    impl<Client> PhactoryApiClient<Client>
    where
        Client: prpc::client::RequestClient,
    {
        pub fn new(client: Client) -> Self {
            Self { client }
        }
        #[doc = " Get basic information about Phactory state."]
        pub async fn get_info(
            &self,
            request: (),
        ) -> Result<super::PhactoryInfo, prpc::client::Error> {
            let response = self
                .client
                .request(
                    "PhactoryAPI.GetInfo",
                    prpc::codec::encode_message_to_vec(&request),
                )
                .await?;
            Ok(prpc::Message::decode(&response[..])?)
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod phactory_api_server {
    use alloc::vec::Vec;
    #[doc = "Generated trait containing RPC methods that should be implemented for use with PhactoryApiServer."]
    pub trait PhactoryApi {
        #[doc = " Get basic information about Phactory state."]
        fn get_info(&self, request: ()) -> Result<super::PhactoryInfo, prpc::server::Error>;
    }
    #[doc = " The greeting service definition."]
    #[derive(Debug)]
    pub struct PhactoryApiServer<T: PhactoryApi> {
        inner: T,
    }
    impl<T: PhactoryApi> PhactoryApiServer<T> {
        pub fn new(inner: T) -> Self {
            Self { inner }
        }
        pub fn dispatch_request(
            &self,
            path: &str,
            data: impl AsRef<[u8]>,
        ) -> Result<Vec<u8>, prpc::server::Error> {
            match path {
                "PhactoryAPI.GetInfo" => {
                    let input: () = prpc::Message::decode(data.as_ref())?;
                    let response = self.inner.get_info(input)?;
                    Ok(prpc::codec::encode_message_to_vec(&response))
                }
                _ => Err(prpc::server::Error::NotFound),
            }
        }
    }
}
