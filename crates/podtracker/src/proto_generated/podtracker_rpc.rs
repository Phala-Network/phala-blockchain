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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPodInfoRequest {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListPodsResponse {
    #[prost(message, repeated, tag = "1")]
    pub pods: ::prost::alloc::vec::Vec<PodInfo>,
}
/// Information about a pod.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PodInfo {
    /// The id given by the creator.
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    /// The image used to create the pod.
    #[prost(string, tag = "2")]
    pub image: ::prost::alloc::string::String,
    /// Container id.
    #[prost(string, tag = "3")]
    pub container_id: ::prost::alloc::string::String,
    /// Portmap for the pod.
    #[prost(message, repeated, tag = "4")]
    pub port_map: ::prost::alloc::vec::Vec<PortMap>,
}
/// A pair of port map
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PortMap {
    #[prost(uint32, tag = "1")]
    pub exposed: u32,
    #[prost(uint32, tag = "2")]
    pub internal: u32,
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
        pub fn status(&self, request: ()) -> Result<super::TrackerInfo, prpc::client::Error> {
            let response = self.client.request(
                "PodtrackerAPI.Status",
                prpc::codec::encode_message_to_vec(&request),
            )?;
            Ok(prpc::Message::decode(&response[..])?)
        }
        #[doc = " Inspect the given pod's status."]
        pub fn get_pod_info(
            &self,
            request: super::GetPodInfoRequest,
        ) -> Result<super::PodInfo, prpc::client::Error> {
            let response = self.client.request(
                "PodtrackerAPI.GetPodInfo",
                prpc::codec::encode_message_to_vec(&request),
            )?;
            Ok(prpc::Message::decode(&response[..])?)
        }
        #[doc = " Inspect the given pod's status."]
        pub fn list_pods(
            &self,
            request: (),
        ) -> Result<super::ListPodsResponse, prpc::client::Error> {
            let response = self.client.request(
                "PodtrackerAPI.ListPods",
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
        &[
            "PodtrackerAPI.Status",
            "PodtrackerAPI.GetPodInfo",
            "PodtrackerAPI.ListPods",
        ]
    }
    #[doc = "Generated trait containing RPC methods that should be implemented for use with PodtrackerApiServer."]
    #[::async_trait::async_trait]
    pub trait PodtrackerApi {
        #[doc = " Get basic information about the podtracker state."]
        async fn status(&mut self, request: ()) -> Result<super::TrackerInfo, prpc::server::Error>;
        #[doc = " Inspect the given pod's status."]
        async fn get_pod_info(
            &mut self,
            request: super::GetPodInfoRequest,
        ) -> Result<super::PodInfo, prpc::server::Error>;
        #[doc = " Inspect the given pod's status."]
        async fn list_pods(
            &mut self,
            request: (),
        ) -> Result<super::ListPodsResponse, prpc::server::Error>;
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
                "PodtrackerAPI.Status" => {
                    let input: () = prpc::Message::decode(data.as_ref())?;
                    let response = self.inner.status(input).await?;
                    Ok(prpc::codec::encode_message_to_vec(&response))
                }
                "PodtrackerAPI.GetPodInfo" => {
                    let input: super::GetPodInfoRequest = prpc::Message::decode(data.as_ref())?;
                    let response = self.inner.get_pod_info(input).await?;
                    Ok(prpc::codec::encode_message_to_vec(&response))
                }
                "PodtrackerAPI.ListPods" => {
                    let input: () = prpc::Message::decode(data.as_ref())?;
                    let response = self.inner.list_pods(input).await?;
                    Ok(prpc::codec::encode_message_to_vec(&response))
                }
                _ => Err(prpc::server::Error::NotFound),
            }
        }
    }
}
