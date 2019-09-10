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
use serde::de;
use serde_json::de::{IoRead, Read, SliceRead, StrRead};
use serde_json::Error;
use std::fmt;
use std::io;

use crate::json::de::{ByteBufVisitor, F32Visitor, F64Visitor};
use std::borrow::Cow;

/// Deserializes a value from a reader of JSON data.
pub fn server_from_reader<R, T>(reader: R) -> Result<T, Error>
where
    R: io::Read,
    T: de::DeserializeOwned,
{
    let mut de = ServerDeserializer::from_reader(reader);
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// Deserializes a value from a string of JSON data.
pub fn server_from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
{
    let mut de = ServerDeserializer::from_str(s);
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// Deserializes a value from a slice of JSON data.
pub fn server_from_slice<'a, T>(s: &'a [u8]) -> Result<T, Error>
where
    T: de::Deserialize<'a>,
{
    let mut de = ServerDeserializer::from_slice(s);
    let value = T::deserialize(&mut de)?;
    de.end()?;
    Ok(value)
}

/// A serde JSON deserializer appropriate for use by Conjure servers.
///
/// In contrast to serde_json, the f32 and f64 types can be deserialized from the strings `"Infinity"`, `"-Infinity"`,
/// and `"NaN"`, and bytes are deserialized from base64 encoded strings. Unknown object fields trigger errors.
pub struct ServerDeserializer<R>(serde_json::Deserializer<R>);

impl<R> ServerDeserializer<IoRead<R>>
where
    R: io::Read,
{
    /// Creates a Conjure JSON server deserializer from an `io::Read`.
    pub fn from_reader(reader: R) -> ServerDeserializer<IoRead<R>> {
        ServerDeserializer(serde_json::Deserializer::from_reader(reader))
    }
}

impl<'a> ServerDeserializer<SliceRead<'a>> {
    /// Creates a Conjure JSON server deserializer from a `&[u8]`.
    pub fn from_slice(bytes: &'a [u8]) -> ServerDeserializer<SliceRead<'a>> {
        ServerDeserializer(serde_json::Deserializer::from_slice(bytes))
    }
}

impl<'a> ServerDeserializer<StrRead<'a>> {
    /// Creates a Conjure JSON server deserializer from a `&str`.
    #[allow(clippy::should_implement_trait)] // match serde_json's API
    pub fn from_str(s: &'a str) -> ServerDeserializer<StrRead<'a>> {
        ServerDeserializer(serde_json::Deserializer::from_str(s))
    }
}

impl<'de, R> ServerDeserializer<R>
where
    R: Read<'de>,
{
    /// Validates that the input stream is at the end or that it only has trailing whitespace.
    pub fn end(&mut self) -> Result<(), Error> {
        self.0.end()
    }
}

macro_rules! delegate_deserialize {
    ($($method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
            where
                V: de::Visitor<'de>
            {
                (self.0).$method(Visitor(visitor))
            }
        )*
    }
}

impl<'a, 'de, R> de::Deserializer<'de> for &'a mut ServerDeserializer<R>
where
    R: Read<'de>,
{
    type Error = Error;

    delegate_deserialize!(
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
        deserialize_i128,
        deserialize_u128,
    );

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_any(F32Visitor(visitor))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_any(F64Visitor(visitor))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_str(ByteBufVisitor(visitor))
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_unit_struct(name, Visitor(visitor))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_newtype_struct(name, Visitor(visitor))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_tuple(len, Visitor(visitor))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_tuple_struct(name, len, Visitor(visitor))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0
            .deserialize_struct(name, fields, StructVisitor { visitor, fields })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_enum(name, variants, Visitor(visitor))
    }

    // we can't delegate this due to the signature, but luckily we know the answer
    fn is_human_readable(&self) -> bool {
        true
    }
}

struct WrapDeserializer<T>(T);

macro_rules! delegate_wrap_deserialize {
    ($($method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, T::Error>
            where
                V: de::Visitor<'de>
            {
                (self.0).$method(Visitor(visitor))
            }
        )*
    }
}

impl<'de, T> de::Deserializer<'de> for WrapDeserializer<T>
where
    T: de::Deserializer<'de>,
{
    type Error = T::Error;

    delegate_wrap_deserialize!(
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
        deserialize_i128,
        deserialize_u128,
    );

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_any(F32Visitor(visitor))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_any(F64Visitor(visitor))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_str(ByteBufVisitor(visitor))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_unit_struct(name, Visitor(visitor))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_newtype_struct(name, Visitor(visitor))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_tuple(len, Visitor(visitor))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_tuple_struct(name, len, Visitor(visitor))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0
            .deserialize_struct(name, fields, StructVisitor { visitor, fields })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_enum(name, variants, Visitor(visitor))
    }

    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

struct Visitor<T>(T);

macro_rules! delegate_visit {
    ($($method:ident = $ty:ty,)*) => {
        $(
            fn $method<E>(self, v: $ty) -> Result<T::Value, E>
            where
                E: de::Error,
            {
                (self.0).$method(v)
            }
        )*
    };
}

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: de::Visitor<'de>,
{
    type Value = T::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.expecting(formatter)
    }

    delegate_visit!(
        visit_bool = bool,
        visit_i8 = i8,
        visit_i16 = i16,
        visit_i32 = i32,
        visit_i64 = i64,
        visit_i128 = i128,
        visit_u8 = u8,
        visit_u16 = u16,
        visit_u32 = u32,
        visit_u64 = u64,
        visit_u128 = u128,
        visit_f32 = f32,
        visit_f64 = f64,
        visit_char = char,
        visit_str = &str,
        visit_borrowed_str = &'de str,
        visit_string = String,
        visit_bytes = &[u8],
        visit_borrowed_bytes = &'de [u8],
        visit_byte_buf = Vec<u8>,
    );

    fn visit_none<E>(self) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        self.0.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.0.visit_some(WrapDeserializer(deserializer))
    }

    fn visit_unit<E>(self) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        self.0.visit_unit()
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.0.visit_newtype_struct(WrapDeserializer(deserializer))
    }

    fn visit_seq<A>(self, seq: A) -> Result<T::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.0.visit_seq(SeqAccess(seq))
    }

    fn visit_map<A>(self, map: A) -> Result<T::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.0.visit_map(MapAccess(map))
    }

    fn visit_enum<A>(self, data: A) -> Result<T::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.0.visit_enum(EnumAccess(data))
    }
}

struct StructVisitor<T> {
    visitor: T,
    fields: &'static [&'static str],
}

macro_rules! delegate_struct_visit {
    ($($method:ident = $ty:ty,)*) => {
        $(
            fn $method<E>(self, v: $ty) -> Result<T::Value, E>
            where
                E: de::Error,
            {
                self.visitor.$method(v)
            }
        )*
    };
}

impl<'de, T> de::Visitor<'de> for StructVisitor<T>
where
    T: de::Visitor<'de>,
{
    type Value = T::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.visitor.expecting(formatter)
    }

    delegate_struct_visit!(
        visit_bool = bool,
        visit_i8 = i8,
        visit_i16 = i16,
        visit_i32 = i32,
        visit_i64 = i64,
        visit_i128 = i128,
        visit_u8 = u8,
        visit_u16 = u16,
        visit_u32 = u32,
        visit_u64 = u64,
        visit_u128 = u128,
        visit_f32 = f32,
        visit_f64 = f64,
        visit_char = char,
        visit_str = &str,
        visit_borrowed_str = &'de str,
        visit_string = String,
        visit_bytes = &[u8],
        visit_borrowed_bytes = &'de [u8],
        visit_byte_buf = Vec<u8>,
    );

    fn visit_none<E>(self) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        self.visitor.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.visitor.visit_some(WrapDeserializer(deserializer))
    }

    fn visit_unit<E>(self) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        self.visitor.visit_unit()
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.visitor
            .visit_newtype_struct(WrapDeserializer(deserializer))
    }

    fn visit_seq<A>(self, seq: A) -> Result<T::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.visitor.visit_seq(SeqAccess(seq))
    }

    fn visit_map<A>(self, map: A) -> Result<T::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.visitor.visit_map(StructMapAccess {
            map,
            fields: self.fields,
            key: None,
        })
    }

    fn visit_enum<A>(self, data: A) -> Result<T::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.visitor.visit_enum(EnumAccess(data))
    }
}

struct SeqAccess<T>(T);

impl<'de, T> de::SeqAccess<'de> for SeqAccess<T>
where
    T: de::SeqAccess<'de>,
{
    type Error = T::Error;

    fn next_element_seed<U>(&mut self, seed: U) -> Result<Option<U::Value>, T::Error>
    where
        U: de::DeserializeSeed<'de>,
    {
        self.0.next_element_seed(DeserializeSeed(seed))
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct MapAccess<T>(T);

impl<'de, T> de::MapAccess<'de> for MapAccess<T>
where
    T: de::MapAccess<'de>,
{
    type Error = T::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, T::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.0.next_key_seed(DeserializeSeed(seed))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, T::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.0.next_value_seed(DeserializeSeed(seed))
    }

    #[allow(clippy::type_complexity)]
    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, T::Error>
    where
        K: de::DeserializeSeed<'de>,
        V: de::DeserializeSeed<'de>,
    {
        self.0
            .next_entry_seed(DeserializeSeed(kseed), DeserializeSeed(vseed))
    }

    fn size_hint(&self) -> Option<usize> {
        self.0.size_hint()
    }
}

struct StructMapAccess<'de, T> {
    map: T,
    fields: &'static [&'static str],
    key: Option<Cow<'de, str>>,
}

impl<'de, T> de::MapAccess<'de> for StructMapAccess<'de, T>
where
    T: de::MapAccess<'de>,
{
    type Error = T::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, T::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.key = None;
        self.map.next_key_seed(KeyDeserializeSeed {
            seed,
            key: &mut self.key,
        })
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, T::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.map.next_value_seed(ValueDeserializeSeed {
            seed,
            fields: self.fields,
            key: &self.key,
        })
    }

    fn size_hint(&self) -> Option<usize> {
        self.map.size_hint()
    }
}

struct EnumAccess<T>(T);

impl<'de, T> de::EnumAccess<'de> for EnumAccess<T>
where
    T: de::EnumAccess<'de>,
{
    type Error = T::Error;
    type Variant = VariantAccess<T::Variant>;

    #[allow(clippy::type_complexity)]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantAccess<T::Variant>), T::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.0
            .variant_seed(DeserializeSeed(seed))
            .map(|(value, variant)| (value, VariantAccess(variant)))
    }
}

struct VariantAccess<T>(T);

impl<'de, T> de::VariantAccess<'de> for VariantAccess<T>
where
    T: de::VariantAccess<'de>,
{
    type Error = T::Error;

    fn unit_variant(self) -> Result<(), T::Error> {
        self.0.unit_variant()
    }

    fn newtype_variant_seed<U>(self, seed: U) -> Result<U::Value, T::Error>
    where
        U: de::DeserializeSeed<'de>,
    {
        self.0.newtype_variant_seed(DeserializeSeed(seed))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.tuple_variant(len, Visitor(visitor))
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.struct_variant(fields, Visitor(visitor))
    }
}

struct DeserializeSeed<T>(T);

impl<'de, T> de::DeserializeSeed<'de> for DeserializeSeed<T>
where
    T: de::DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.0.deserialize(WrapDeserializer(deserializer))
    }
}

struct KeyDeserializeSeed<'de, 'a, T> {
    seed: T,
    key: &'a mut Option<Cow<'de, str>>,
}

impl<'de, 'a, T> de::DeserializeSeed<'de> for KeyDeserializeSeed<'de, 'a, T>
where
    T: de::DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.seed.deserialize(KeyDeserializer {
            deserializer,
            key: self.key,
        })
    }
}

struct KeyDeserializer<'de, 'a, T> {
    deserializer: T,
    key: &'a mut Option<Cow<'de, str>>,
}

macro_rules! delegate_key_deserialize {
    ($($method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, T::Error>
            where
                V: de::Visitor<'de>
            {
                self.deserializer.$method(KeyVisitor { visitor, key: self.key })
            }
        )*
    }
}

impl<'de, 'a, T> de::Deserializer<'de> for KeyDeserializer<'de, 'a, T>
where
    T: de::Deserializer<'de>,
{
    type Error = T::Error;

    delegate_key_deserialize!(
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
        deserialize_i128,
        deserialize_u128,
    );

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_any(F32Visitor(visitor))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_any(F64Visitor(visitor))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_str(ByteBufVisitor(visitor))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_unit_struct(
            name,
            KeyVisitor {
                visitor,
                key: self.key,
            },
        )
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_newtype_struct(
            name,
            KeyVisitor {
                visitor,
                key: self.key,
            },
        )
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_tuple(
            len,
            KeyVisitor {
                visitor,
                key: self.key,
            },
        )
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_tuple_struct(
            name,
            len,
            KeyVisitor {
                visitor,
                key: self.key,
            },
        )
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        // FIXME this is a bit awkward...
        self.deserializer
            .deserialize_struct(name, fields, StructVisitor { visitor, fields })
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_enum(
            name,
            variants,
            KeyVisitor {
                visitor,
                key: self.key,
            },
        )
    }

    fn is_human_readable(&self) -> bool {
        self.deserializer.is_human_readable()
    }
}

struct KeyVisitor<'de, 'a, T> {
    visitor: T,
    key: &'a mut Option<Cow<'de, str>>,
}

impl<'de, 'a, T> de::Visitor<'de> for KeyVisitor<'de, 'a, T>
where
    T: de::Visitor<'de>,
{
    type Value = T::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.visitor.expecting(formatter)
    }

    delegate_struct_visit!(
        visit_bool = bool,
        visit_i8 = i8,
        visit_i16 = i16,
        visit_i32 = i32,
        visit_i64 = i64,
        visit_i128 = i128,
        visit_u8 = u8,
        visit_u16 = u16,
        visit_u32 = u32,
        visit_u64 = u64,
        visit_u128 = u128,
        visit_f32 = f32,
        visit_f64 = f64,
        visit_char = char,
        visit_bytes = &[u8],
        visit_borrowed_bytes = &'de [u8],
        visit_byte_buf = Vec<u8>,
    );

    fn visit_str<E>(self, value: &str) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        *self.key = Some(Cow::Owned(value.to_string()));
        self.visitor.visit_str(value)
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        *self.key = Some(Cow::Borrowed(value));
        self.visitor.visit_borrowed_str(value)
    }

    fn visit_string<E>(self, value: String) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        *self.key = Some(Cow::Owned(value.to_string()));
        self.visitor.visit_string(value)
    }

    fn visit_none<E>(self) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        self.visitor.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.visitor.visit_some(WrapDeserializer(deserializer))
    }

    fn visit_unit<E>(self) -> Result<T::Value, E>
    where
        E: de::Error,
    {
        self.visitor.visit_unit()
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.visitor
            .visit_newtype_struct(WrapDeserializer(deserializer))
    }

    fn visit_seq<A>(self, seq: A) -> Result<T::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.visitor.visit_seq(SeqAccess(seq))
    }

    fn visit_map<A>(self, map: A) -> Result<T::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.visitor.visit_map(MapAccess(map))
    }

    fn visit_enum<A>(self, data: A) -> Result<T::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.visitor.visit_enum(EnumAccess(data))
    }
}

struct ValueDeserializeSeed<'de, 'a, T> {
    seed: T,
    fields: &'static [&'static str],
    key: &'a Option<Cow<'de, str>>,
}

impl<'de, 'a, T> de::DeserializeSeed<'de> for ValueDeserializeSeed<'de, 'a, T>
where
    T: de::DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<T::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        self.seed.deserialize(ValueDeserializer {
            deserializer,
            fields: self.fields,
            key: self.key,
        })
    }
}

struct ValueDeserializer<'de, 'a, T> {
    deserializer: T,
    fields: &'static [&'static str],
    key: &'a Option<Cow<'de, str>>,
}

macro_rules! delegate_value_deserialize {
    ($($method:ident,)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, T::Error>
            where
                V: de::Visitor<'de>
            {
                self.deserializer.$method(Visitor(visitor))
            }
        )*
    }
}

impl<'de, 'a, T> de::Deserializer<'de> for ValueDeserializer<'de, 'a, T>
where
    T: de::Deserializer<'de>,
{
    type Error = T::Error;

    delegate_value_deserialize!(
        deserialize_any,
        deserialize_bool,
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_i128,
        deserialize_u128,
    );

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_any(F32Visitor(visitor))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_any(F64Visitor(visitor))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_str(ByteBufVisitor(visitor))
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_unit_struct(name, Visitor(visitor))
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_newtype_struct(name, Visitor(visitor))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer.deserialize_tuple(len, Visitor(visitor))
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_tuple_struct(name, len, Visitor(visitor))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_struct(name, fields, Visitor(visitor))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_enum(name, variants, Visitor(visitor))
    }

    fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value, T::Error>
    where
        V: de::Visitor<'de>,
    {
        let key = match self.key {
            Some(key) => &**key,
            None => "<unknown>",
        };

        Err(de::Error::unknown_field(key, self.fields))
    }

    fn is_human_readable(&self) -> bool {
        self.deserializer.is_human_readable()
    }
}
