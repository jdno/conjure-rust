// Copyright 2019 Palantir Technologies, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use proc_macro2::TokenStream;
use quote::quote;

use crate::context::Context;
use crate::types::{
    ArgumentDefinition, AuthType, EndpointDefinition, ParameterType, ServiceDefinition, Type,
};

pub fn generate(ctx: &Context, def: &ServiceDefinition) -> TokenStream {
    let docs = ctx.docs(def.docs());
    let name = ctx.type_name(&format!("{}Client", def.service_name().name()));

    let endpoints = def
        .endpoints()
        .iter()
        .map(|e| generate_endpoint(ctx, def, e));

    quote! {
        #docs
        #[derive(Clone, Debug)]
        pub struct #name<T>(T);

        impl<T> #name<T>
        where
            T: conjure_http::client::Client
        {
            /// Creates a new client.
            #[inline]
            pub fn new(client: T) -> #name<T> {
                #name(client)
            }

            #(#endpoints)*
        }
    }
}

fn generate_endpoint(
    ctx: &Context,
    def: &ServiceDefinition,
    endpoint: &EndpointDefinition,
) -> TokenStream {
    let docs = ctx.docs(endpoint.docs());
    let deprecated = match endpoint.deprecated() {
        Some(docs) => {
            let docs = &**docs;
            quote! {
                #[deprecated(note = #docs)]
            }
        }
        None => quote!(),
    };
    let name = ctx.field_name(endpoint.endpoint_name());

    let body_arg = body_arg(endpoint);
    let params = params(ctx, body_arg);

    let auth = quote!(auth_);
    let auth_arg = auth_arg(endpoint, &auth);
    let args = endpoint.args().iter().map(|a| {
        let name = ctx.field_name(a.arg_name());
        let ty = arg_type(ctx, def, a);
        quote!(#name: #ty)
    });

    let ret = return_type(ctx, endpoint);
    let ret_name = return_type_name(ctx, def, &ret);
    let where_ = where_(ctx, body_arg);

    let method = endpoint
        .http_method()
        .as_str()
        .parse::<TokenStream>()
        .unwrap();

    let path = &**endpoint.http_path();

    let setup_body = setup_body(ctx, body_arg);
    let body = generate_body(ctx, body_arg);

    let path_params = quote!(path_params_);
    let setup_path_params = setup_path_params(ctx, endpoint, &path_params);

    let query_params = quote!(query_params_);
    let setup_query_params = setup_query_params(ctx, endpoint, &query_params);

    let headers = quote!(headers_);
    let setup_headers = setup_headers(ctx, endpoint, &headers, &auth, body_arg, &ret);

    let request = quote! {
        self.0.request(
            conjure_http::private::http::Method::#method,
            #path,
            #path_params,
            #query_params,
            #headers,
            #body,
        )
    };

    let handle_response = handle_response(ctx, &ret, &request);

    quote! {
        #docs
        #deprecated
        pub fn #name #params(&self #auth_arg #(, #args)*) -> Result<#ret_name, conjure_http::private::Error>
        #where_
        {
            #setup_body
            #setup_path_params
            #setup_query_params
            #setup_headers
            #handle_response
        }
    }
}

fn body_arg(endpoint: &EndpointDefinition) -> Option<&ArgumentDefinition> {
    endpoint.args().iter().find(|a| match a.param_type() {
        ParameterType::Body(_) => true,
        _ => false,
    })
}

fn params(ctx: &Context, body_arg: Option<&ArgumentDefinition>) -> TokenStream {
    match body_arg {
        Some(a) if ctx.is_binary(a.type_()) => quote!(<U>),
        _ => quote!(),
    }
}

fn where_(ctx: &Context, body_arg: Option<&ArgumentDefinition>) -> TokenStream {
    match body_arg {
        Some(a) if ctx.is_binary(a.type_()) => quote!(where U: conjure_http::client::IntoWriteBody),
        _ => quote!(),
    }
}

fn auth_arg(endpoint: &EndpointDefinition, auth: &TokenStream) -> TokenStream {
    match endpoint.auth() {
        Some(_) => quote!(, #auth: &conjure_object::BearerToken),
        None => quote!(),
    }
}

fn arg_type(ctx: &Context, def: &ServiceDefinition, arg: &ArgumentDefinition) -> TokenStream {
    if ctx.is_binary(arg.type_()) {
        quote!(U)
    } else {
        ctx.borrowed_rust_type(def.service_name(), arg.type_())
    }
}

fn return_type<'a>(ctx: &Context, endpoint: &'a EndpointDefinition) -> ReturnType<'a> {
    match endpoint.returns() {
        Some(ret) => match ctx.is_optional(ret) {
            Some(inner) if ctx.is_binary(inner) => ReturnType::OptionalBinary,
            _ if ctx.is_binary(ret) => ReturnType::Binary,
            _ => ReturnType::Json(ret),
        },
        None => ReturnType::None,
    }
}

fn return_type_name(ctx: &Context, def: &ServiceDefinition, ty: &ReturnType<'_>) -> TokenStream {
    match ty {
        ReturnType::None => quote!(()),
        ReturnType::Json(ty) => ctx.rust_type(def.service_name(), ty),
        ReturnType::Binary => quote!(T::ResponseBody),
        ReturnType::OptionalBinary => quote!(Option<T::ResponseBody>),
    }
}

fn setup_body(ctx: &Context, body: Option<&ArgumentDefinition>) -> TokenStream {
    match body {
        Some(body) if ctx.is_binary(body.type_()) => {
            let name = ctx.field_name(body.arg_name());
            quote! {
                let mut #name = #name.into_write_body();
            }
        }
        _ => quote!(),
    }
}

fn generate_body(ctx: &Context, body: Option<&ArgumentDefinition>) -> TokenStream {
    let body = match body {
        Some(body) => body,
        None => return quote!(conjure_http::client::Body::Empty),
    };

    let name = ctx.field_name(body.arg_name());
    if ctx.is_binary(body.type_()) {
        quote! {
            conjure_http::client::Body::Streaming(&mut #name)
        }
    } else {
        quote! {
            conjure_http::client::Body::Fixed(
                conjure_http::private::json::to_vec(&#name)
                    .map_err(conjure_http::private::Error::internal)?,
            )
        }
    }
}

fn setup_path_params(
    ctx: &Context,
    endpoint: &EndpointDefinition,
    path_params: &TokenStream,
) -> TokenStream {
    let mut parameters = vec![];

    for argument in endpoint.args() {
        match argument.param_type() {
            ParameterType::Path(_) => {}
            _ => continue,
        }

        let key = &**argument.arg_name();
        let name = ctx.field_name(key);

        let parameter = quote! {
            #path_params.insert(
                #key,
                conjure_object::ToPlain::to_plain(&#name),
            );
        };
        parameters.push(parameter);
    }

    let mutability = if parameters.is_empty() {
        quote!()
    } else {
        quote!(mut)
    };

    quote! {
        let #mutability #path_params = conjure_http::PathParams::new();
        #(#parameters)*
    }
}

fn setup_query_params(
    ctx: &Context,
    endpoint: &EndpointDefinition,
    query_params: &TokenStream,
) -> TokenStream {
    let mut parameters = vec![];

    for argument in endpoint.args() {
        let query = match argument.param_type() {
            ParameterType::Query(query) => query,
            _ => continue,
        };

        let key = &**query.param_id();
        let name = ctx.field_name(argument.arg_name());

        let parameter = if ctx.is_iterable(argument.type_()) {
            quote! {
                #query_params.insert_all(
                    #key,
                    #name.iter().map(conjure_object::ToPlain::to_plain),
                );
            }
        } else {
            quote! {
                #query_params.insert(
                    #key,
                    conjure_object::ToPlain::to_plain(&#name),
                );
            }
        };
        parameters.push(parameter);
    }

    let mutability = if parameters.is_empty() {
        quote!()
    } else {
        quote!(mut)
    };

    quote! {
        let #mutability #query_params = conjure_http::QueryParams::new();
        #(#parameters)*
    }
}

fn setup_headers(
    ctx: &Context,
    endpoint: &EndpointDefinition,
    headers: &TokenStream,
    auth: &TokenStream,
    body: Option<&ArgumentDefinition>,
    response: &ReturnType<'_>,
) -> TokenStream {
    let mut parameters = vec![];

    if let Some(parameter) = auth_header(endpoint, headers, auth) {
        parameters.push(parameter);
    }
    if let Some(parameter) = content_type_header(ctx, body, headers) {
        parameters.push(parameter);
    }
    if let Some(parameter) = accept_header(response, headers) {
        parameters.push(parameter);
    }

    for argument in endpoint.args() {
        let header = match argument.param_type() {
            ParameterType::Header(header) => header,
            _ => continue,
        };

        // HeaderName::from_static expects http2-style lowercased headers
        let header = header.param_id().to_lowercase();
        let name = ctx.field_name(argument.arg_name());

        let mut parameter = quote! {
            #headers.insert(
                conjure_http::private::http::header::HeaderName::from_static(#header),
                conjure_http::private::http::header::HeaderValue::from_shared(
                    conjure_object::ToPlain::to_plain(&#name).into(),
                ).map_err(conjure_http::private::Error::internal_safe)?,
            );
        };

        // this is kind of dubious since the only iterable header parameter types are optionals, but it's a PITA to
        // match on an aliased option so we'll just iterate.
        if ctx.is_iterable(argument.type_()) {
            parameter = quote! {
                for #name in #name {
                    #parameter
                }
            }
        }

        parameters.push(parameter);
    }

    let mutability = if parameters.is_empty() {
        quote!()
    } else {
        quote!(mut)
    };

    quote! {
        let #mutability #headers = conjure_http::private::http::HeaderMap::new();
        #(#parameters)*
    }
}

fn auth_header(
    endpoint: &EndpointDefinition,
    headers: &TokenStream,
    auth: &TokenStream,
) -> Option<TokenStream> {
    let (header, template) = match endpoint.auth() {
        Some(AuthType::Cookie(cookie)) => {
            (quote!(COOKIE), format!("{}={{}}", cookie.cookie_name()))
        }
        Some(AuthType::Header(_)) => (quote!(AUTHORIZATION), "Bearer {}".to_string()),
        None => return None,
    };

    let parameter = quote! {
        #headers.insert(
            conjure_http::private::http::header::#header,
            conjure_http::private::http::header::HeaderValue::from_shared(
                format!(#template, #auth.as_str()).into(),
            ).expect("bearer tokens are valid headers"),
        );
    };
    Some(parameter)
}

fn content_type_header(
    ctx: &Context,
    body: Option<&ArgumentDefinition>,
    headers: &TokenStream,
) -> Option<TokenStream> {
    let body = body?;

    let content_type = if ctx.is_binary(body.type_()) {
        "application/octet-stream"
    } else {
        "application/json"
    };

    let parameter = quote! {
        #headers.insert(
            conjure_http::private::http::header::CONTENT_TYPE,
            conjure_http::private::http::header::HeaderValue::from_static(#content_type),
        );
    };
    Some(parameter)
}

fn accept_header(response: &ReturnType<'_>, headers: &TokenStream) -> Option<TokenStream> {
    let content_type = match response {
        ReturnType::None => return None,
        ReturnType::Json(_) => "application/json",
        ReturnType::Binary | ReturnType::OptionalBinary => "application/octet-stream",
    };

    let parameter = quote! {
        #headers.insert(
            conjure_http::private::http::header::ACCEPT,
            conjure_http::private::http::header::HeaderValue::from_static(#content_type),
        );
    };
    Some(parameter)
}

fn handle_response(ctx: &Context, ty: &ReturnType<'_>, response: &TokenStream) -> TokenStream {
    match ty {
        ReturnType::None => {
            quote! {
                #response?;
                Ok(())
            }
        }
        ReturnType::Json(ty) => {
            let mut convert = quote! {
                conjure_http::private::json::client_from_reader(response.body_mut())
                    .map_err(conjure_http::private::Error::internal)
            };
            if ctx.is_iterable(ty) {
                convert = quote! {
                    if response.status() == conjure_http::private::http::StatusCode::NO_CONTENT {
                        Ok(Default::default())
                    } else {
                        #convert
                    }
                };
            }

            quote! {
                let mut response = #response?;
                #convert
            }
        }
        ReturnType::Binary => {
            quote! {
                let response = #response?;
                Ok(response.into_body())
            }
        }
        ReturnType::OptionalBinary => {
            quote! {
                let response = #response?;
                if response.status() == conjure_http::private::http::StatusCode::NO_CONTENT {
                    Ok(None)
                } else {
                    Ok(Some(response.into_body()))
                }
            }
        }
    }
}

enum ReturnType<'a> {
    None,
    Json(&'a Type),
    Binary,
    OptionalBinary,
}
