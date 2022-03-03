/// Basic information about the podtracker state.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TrackerInfo {
    /// Current number of pods.
    #[prost(uint32, tag = "1")]
    pub pods_running: u32,
    /// Total allocated historically.
    #[prost(uint32, tag = "2")]
    pub pods_allocated: u32,
    /// How many TCP ports are currently available.
    #[prost(uint32, tag = "3")]
    pub tcp_ports_available: u32,
}
#[doc = r" Generated client implementations."]
pub mod podtracker_api_client {
    #[doc = " The podtracket control RPC definition."]
    #[derive(Debug)]
    pub struct PodtrackerApiClient<Client> {
        client: Client,
    }
    impl<Client> PodtrackerApiClient<Client>
    where
        Client: prpc::client::RequestClient,
    {
        pub fn new(client: Client) -> Self {
            Self { client }
        }
        #[doc = " Get basic information about the podtracker state."]
        pub fn get_info(&self, request: ()) -> Result<super::TrackerInfo, prpc::client::Error> {
            let response = self.client.request(
                "PodtrackerAPI.GetInfo",
                prpc::codec::encode_message_to_vec(&request),
            )?;
            Ok(prpc::Message::decode(&response[..])?)
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod podtracker_api_server {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    pub fn supported_methods() -> &'static [&'static str] {
        &["PodtrackerAPI.GetInfo"]
    }
    #[doc = "Generated trait containing RPC methods that should be implemented for use with PodtrackerApiServer."]
    #[::async_trait::async_trait]
    pub trait PodtrackerApi {
        #[doc = " Get basic information about the podtracker state."]
        async fn get_info(
            &mut self,
            request: (),
        ) -> Result<super::TrackerInfo, prpc::server::Error>;
    }
    #[doc = " The podtracket control RPC definition."]
    #[derive(Debug)]
    pub struct PodtrackerApiServer<T: PodtrackerApi> {
        inner: T,
    }
    impl<T: PodtrackerApi> PodtrackerApiServer<T> {
        pub fn new(inner: T) -> Self {
            Self { inner }
        }
        pub async fn dispatch_request(
            &mut self,
            path: &str,
            data: impl AsRef<[u8]>,
        ) -> Result<Vec<u8>, prpc::server::Error> {
            match path {
                "PodtrackerAPI.GetInfo" => {
                    let input: () = prpc::Message::decode(data.as_ref())?;
                    let response = self.inner.get_info(input).await?;
                    Ok(prpc::codec::encode_message_to_vec(&response))
                }
                _ => Err(prpc::server::Error::NotFound),
            }
        }
    }
}
