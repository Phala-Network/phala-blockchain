use super::{Attributes, Method, Service};
use crate::{generate_doc_comments, naive_snake_case};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate service for client.
///
/// This takes some `Service` and will generate a `TokenStream` that contains
/// a public module with the generated client.
pub fn generate<T: Service>(
    service: &T,
    emit_package: bool,
    proto_path: &str,
    compile_well_known_types: bool,
    attributes: &Attributes,
) -> TokenStream {
    let service_ident = quote::format_ident!("{}Client", service.name());
    let client_mod = quote::format_ident!("{}_client", naive_snake_case(service.name()));
    let methods = generate_methods(service, emit_package, proto_path, compile_well_known_types);

    let service_doc = generate_doc_comments(service.comment());

    let package = if emit_package { service.package() } else { "" };
    let path = format!(
        "{}{}{}",
        package,
        if package.is_empty() { "" } else { "." },
        service.identifier()
    );

    let mod_attributes = attributes.for_mod(package);
    let struct_attributes = attributes.for_struct(&path);

    quote! {
        /// Generated client implementations.
        #(#mod_attributes)*
        pub mod #client_mod {
            #service_doc
            #(#struct_attributes)*
            #[derive(Debug)]
            pub struct #service_ident<Client> {
                client: Client,
            }

            impl<Client> #service_ident<Client>
            where
                Client: prpc::client::RequestClient
            {
                pub fn new(client: Client) -> Self {
                    Self { client }
                }

                #methods
            }
        }
    }
}

fn generate_methods<T: Service>(
    service: &T,
    emit_package: bool,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods() {
        let path = crate::join_path(
            emit_package,
            service.package(),
            service.identifier(),
            method.identifier(),
        );

        stream.extend(generate_doc_comments(method.comment()));

        let method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary(method, proto_path, compile_well_known_types, path),
            _ => {
                panic!("Only unary method supported");
            }
        };

        stream.extend(method);
    }

    stream
}

fn generate_unary<T: Method>(
    method: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    path: String,
) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        pub async fn #ident(
            &self,
            request: #request,
        ) -> Result<#response, prpc::client::Error> {
            let response = self.client.request(#path, prpc::codec::encode_message_to_vec(&request)).await?;
            Ok(prpc::Message::decode(&response[..])?)
        }
    }
}
