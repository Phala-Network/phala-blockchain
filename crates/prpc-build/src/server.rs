use super::{Attributes, Method, Service};
use crate::{generate_doc_comment, generate_doc_comments, naive_snake_case};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, Lit, LitStr};

/// Generate service for Server.
///
/// This takes some `Service` and will generate a `TokenStream` that contains
/// a public module containing the server service and handler trait.
pub fn generate<T: Service>(
    service: &T,
    emit_package: bool,
    proto_path: &str,
    compile_well_known_types: bool,
    attributes: &Attributes,
) -> TokenStream {
    let methods = generate_methods(service, proto_path, emit_package, compile_well_known_types);

    let server_service = quote::format_ident!("{}Server", service.name());
    let server_trait = quote::format_ident!("{}", service.name());
    let server_mod = quote::format_ident!("{}_server", naive_snake_case(&service.name()));
    let generated_trait = generate_trait(
        service,
        proto_path,
        compile_well_known_types,
        server_trait.clone(),
    );
    let service_doc = generate_doc_comments(service.comment());
    let package = if emit_package { service.package() } else { "" };
    let path = crate::join_path(emit_package, service.package(), service.identifier(), "");
    let mod_attributes = attributes.for_mod(package);
    let struct_attributes = attributes.for_struct(&path);

    quote! {
        /// Generated server implementations.
        #(#mod_attributes)*
        pub mod #server_mod {
            #generated_trait

            #service_doc
            #(#struct_attributes)*
            #[derive(Debug)]
            pub struct #server_service<T: #server_trait> {
                inner: T,
            }

            impl<T: #server_trait> #server_service<T> {
                pub fn new(inner: T) -> Self {
                    Self {
                        inner,
                    }
                }

                fn dispatch_request(&self, path: &str, data: Vec<u8>) -> Result<Vec<u8>, prpc::server::Error> {
                    match path {
                        #methods
                        _ => Err(prpc::server::Error::NotFound),
                    }
                }
            }
        }
    }
}

fn generate_trait<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
    server_trait: Ident,
) -> TokenStream {
    let methods = generate_trait_methods(service, proto_path, compile_well_known_types);
    let trait_doc = generate_doc_comment(&format!(
        "Generated trait containing RPC methods that should be implemented for use with {}Server.",
        service.name()
    ));

    quote! {
        #trait_doc
        pub trait #server_trait {
            #methods
        }
    }
}

fn generate_trait_methods<T: Service>(
    service: &T,
    proto_path: &str,
    compile_well_known_types: bool,
) -> TokenStream {
    let mut stream = TokenStream::new();

    for method in service.methods() {
        let name = quote::format_ident!("{}", method.name());

        let (req_message, res_message) =
            method.request_response_name(proto_path, compile_well_known_types);

        let method_doc = generate_doc_comments(method.comment());

        let method = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => {
                quote! {
                    #method_doc
                    fn #name(&self, request: #req_message)
                        -> Result<#res_message, prpc::server::Error>;
                }
            }
            _ => {
                panic!("Streaming RPC not supported");
            }
        };

        stream.extend(method);
    }

    stream
}

fn generate_methods<T: Service>(
    service: &T,
    proto_path: &str,
    emit_package: bool,
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
        let method_path = Lit::Str(LitStr::new(&path, Span::call_site()));
        let method_ident = quote::format_ident!("{}", method.name());
        let server_trait = quote::format_ident!("{}", service.name());

        let method_stream = match (method.client_streaming(), method.server_streaming()) {
            (false, false) => generate_unary(
                method,
                proto_path,
                compile_well_known_types,
                method_ident,
                server_trait,
            ),
            _ => {
                panic!("Streaming RPC not supported");
            }
        };

        let method = quote! {
            #method_path => {
                #method_stream
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
    method_ident: Ident,
    _server_trait: Ident,
) -> TokenStream {
    let (request, _response) = method.request_response_name(proto_path, compile_well_known_types);

    quote! {
        let input: #request = prpc::Message::decode(&data[..])?;
        let response = self.inner.#method_ident(input)?;
        Ok(prpc::codec::encode_message_to_vec(&response))
    }
}
