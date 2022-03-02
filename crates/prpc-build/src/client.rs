use super::{Method, Service};
use crate::{generate_doc_comments, naive_snake_case, Builder};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

/// Generate service for client.
///
/// This takes some `Service` and will generate a `TokenStream` that contains
/// a public module with the generated client.
pub fn generate<T: Service>(service: &T, builder: &Builder) -> TokenStream {
    let service_ident = quote::format_ident!("{}Client", service.name());
    let client_mod = quote::format_ident!("{}_client", naive_snake_case(service.name()));
    let methods = generate_methods(service, builder);

    let service_doc = generate_doc_comments(service.comment());

    let package = if builder.emit_package {
        service.package()
    } else {
        ""
    };
    let path = format!(
        "{}{}{}",
        package,
        if package.is_empty() { "" } else { "." },
        service.identifier()
    );

    let mod_attributes = builder.client_attributes.for_mod(package);
    let struct_attributes = builder.client_attributes.for_struct(&path);

    let http_client: TokenStream = if builder.async_client {
        quote! {
            prpc::client::AsyncRequestClient
        }
    } else {
        quote! {
            prpc::client::RequestClient
        }
    };

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
                Client: #http_client
            {
                pub fn new(client: Client) -> Self {
                    Self { client }
                }

                #methods
            }
        }
    }
}

fn generate_methods<T: Service>(service: &T, builder: &Builder) -> TokenStream {
    let mut stream = TokenStream::new();
    for method in service.methods() {
        let path = crate::join_path(
            builder.emit_package,
            service.package(),
            service.identifier(),
            method.identifier(),
        );

        stream.extend(generate_doc_comments(method.comment()));

        let method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary(method, path, builder),
            _ => {
                panic!("Only unary method supported");
            }
        };

        stream.extend(method);
    }

    stream
}

fn generate_unary<T: Method>(method: &T, path: String, builder: &Builder) -> TokenStream {
    let ident = format_ident!("{}", method.name());
    let (request, response) =
        method.request_response_name(&builder.proto_path, builder.compile_well_known_types);

    let async_or_not = if builder.async_client {
        quote! {
            async
        }
    } else {
        quote! {}
    };

    let await_or_not = if builder.async_client {
        quote! {
            .await
        }
    } else {
        quote! {}
    };

    quote! {
        pub #async_or_not fn #ident(
            &self,
            request: #request,
        ) -> Result<#response, prpc::client::Error> {
            let response = self.client.request(#path, prpc::codec::encode_message_to_vec(&request)) #await_or_not ?;
            Ok(prpc::Message::decode(&response[..])?)
        }
    }
}
