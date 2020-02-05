// Copyright 2018 Palantir Technologies, Inc.
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
use crate::types::{EnumDefinition, EnumValueDefinition};

pub fn generate(ctx: &Context, def: &EnumDefinition) -> TokenStream {
    let enum_ = generate_enum(ctx, def);
    let unknown = generate_unknown(ctx, def);

    quote! {
        use conjure_object::serde::{ser, de};
        use std::fmt;
        use std::str;

        #enum_
        #unknown
    }
}

fn generate_enum(ctx: &Context, def: &EnumDefinition) -> TokenStream {
    let root_docs = ctx.docs(def.docs());
    let name = ctx.type_name(def.type_name().name());
    let result = ctx.result_ident(def.type_name());
    let ok = ctx.ok_ident(def.type_name());
    let err = ctx.err_ident(def.type_name());
    let unknown = unknown(ctx, def);

    let variants = def.values().iter().map(|v| {
        let docs = ctx.docs(v.docs());
        let deprecated = ctx.deprecated(v.deprecated());
        let name = ctx.type_name(v.value());
        quote! {
            #docs
            #deprecated
            #name,
        }
    });

    let other_variant = if ctx.exhaustive() {
        quote!()
    } else {
        quote! {
            /// An unknown variant.
            Unknown(#unknown)
        }
    };

    let as_str_arms = def.values().iter().map(|v| {
        let value = v.value();
        let variant = ctx.type_name(v.value());
        let allow_deprecated = ctx.allow_deprecated(v.deprecated());
        quote! {
            #allow_deprecated
            #name::#variant => #value,
        }
    });

    let as_str_other = if ctx.exhaustive() {
        quote!()
    } else {
        quote!(#name::Unknown(v) => &*v,)
    };

    let from_str_arms = def.values().iter().map(|v| {
        let value = v.value();
        let variant = ctx.type_name(value);
        let allow_deprecated = ctx.allow_deprecated(v.deprecated());
        quote! {
            #allow_deprecated
            #value => #ok(#name::#variant),
        }
    });

    let from_str_other = if ctx.exhaustive() {
        quote! {
            _ => #err(conjure_object::plain::ParseEnumError::new()),
        }
    } else {
        quote! {
            v => {
                if conjure_object::private::valid_enum_variant(v) {
                    #ok(#name::Unknown(#unknown(v.to_string().into_boxed_str())))
                } else {
                    #err(conjure_object::plain::ParseEnumError::new())
                }
            }
        }
    };

    let values = def.values().iter().map(EnumValueDefinition::value);

    quote! {
        #root_docs
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum #name {
            #(#variants)*
            #other_variant
        }

        impl #name {
            /// Returns the string representation of the enum.
            #[inline]
            pub fn as_str(&self) -> &str {
                match self {
                    #(#as_str_arms)*
                    #as_str_other
                }
            }
        }

        impl fmt::Display for #name {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt::Display::fmt(self.as_str(), fmt)
            }
        }

        impl conjure_object::Plain for #name {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                conjure_object::Plain::fmt(self.as_str(), fmt)
            }
        }

        impl str::FromStr for #name {
            type Err = conjure_object::plain::ParseEnumError;

            #[inline]
            fn from_str(v: &str) -> #result<#name, conjure_object::plain::ParseEnumError> {
                match v {
                    #(#from_str_arms)*
                    #from_str_other
                }
            }
        }

        impl conjure_object::FromPlain for #name {
            type Err = conjure_object::plain::ParseEnumError;

            #[inline]
            fn from_plain(v: &str) -> #result<#name, conjure_object::plain::ParseEnumError> {
                v.parse()
            }
        }

        impl ser::Serialize for #name {
            fn serialize<S>(&self, s: S) -> #result<S::Ok, S::Error>
            where
                S: ser::Serializer,
            {
                s.serialize_str(self.as_str())
            }
        }

        impl<'de> de::Deserialize<'de> for #name {
            fn deserialize<D>(d: D) -> #result<#name, D::Error>
            where
                D: de::Deserializer<'de>
            {
                d.deserialize_str(Visitor_)
            }
        }

        struct Visitor_;

        impl<'de> de::Visitor<'de> for Visitor_ {
            type Value = #name;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> #result<#name, E>
            where
                E: de::Error,
            {
                match v.parse() {
                    #ok(e) => Ok(e),
                    #err(_) => #err(de::Error::unknown_variant(v, &[#(#values, )*])),
                }
            }
        }
    }
}

fn unknown(ctx: &Context, def: &EnumDefinition) -> TokenStream {
    if ctx.type_name(def.type_name().name()) == "Unknown" {
        quote!(Unknown_)
    } else {
        quote!(Unknown)
    }
}

fn generate_unknown(ctx: &Context, def: &EnumDefinition) -> TokenStream {
    if ctx.exhaustive() {
        return quote!();
    }

    let doc = format!(
        "An unknown variant of the {} enum.",
        ctx.type_name(def.type_name().name())
    );

    let unknown = unknown(ctx, def);
    let box_ = ctx.box_ident(def.type_name());

    quote! {
        #[doc = #doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct #unknown(#box_<str>);

        impl std::ops::Deref for #unknown {
            type Target = str;

            #[inline]
            fn deref(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for #unknown {
            #[inline]
            fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, fmt)
            }
        }
    }
}
