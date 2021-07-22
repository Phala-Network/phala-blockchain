/// The request message containing the user's name.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The response message containing the greetings
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloReply {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
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
        #[doc = " Sends a greeting"]
        pub async fn say_hello(
            &self,
            request: super::HelloRequest,
        ) -> Result<super::HelloReply, prpc::client::Error> {
            let response = self
                .client
                .request(
                    "PhactoryAPI.SayHello",
                    prpc::codec::encode_message_to_vec(&request),
                )
                .await?;
            Ok(prpc::Message::decode(&response[..])?)
        }
        #[doc = "/ Get basic information about Phactory state."]
        pub async fn get_info(
            &self,
            request: (),
        ) -> Result<super::HelloReply, prpc::client::Error> {
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
    #[doc = "Generated trait containing RPC methods that should be implemented for use with PhactoryApiServer."]
    pub trait PhactoryApi {
        #[doc = " Sends a greeting"]
        fn say_hello(
            &self,
            request: super::HelloRequest,
        ) -> Result<super::HelloReply, prpc::server::Error>;
        #[doc = "/ Get basic information about Phactory state."]
        fn get_info(&self, request: ()) -> Result<super::HelloReply, prpc::server::Error>;
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
        fn dispatch_request(
            &self,
            path: &str,
            data: Vec<u8>,
        ) -> Result<Vec<u8>, prpc::server::Error> {
            match path {
                "PhactoryAPI.SayHello" => {
                    let input: super::HelloRequest = prpc::Message::decode(&data[..])?;
                    let response = self.inner.say_hello(input)?;
                    Ok(prpc::codec::encode_message_to_vec(&response))
                }
                "PhactoryAPI.GetInfo" => {
                    let input: () = prpc::Message::decode(&data[..])?;
                    let response = self.inner.get_info(input)?;
                    Ok(prpc::codec::encode_message_to_vec(&response))
                }
                _ => Err(prpc::server::Error::NotFound),
            }
        }
    }
}
