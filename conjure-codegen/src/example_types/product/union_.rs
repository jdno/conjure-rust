use conjure_object::private::{UnionField_, UnionTypeField_};
use conjure_object::serde::ser::SerializeMap as SerializeMap_;
use conjure_object::serde::{de, ser};
use std::fmt;
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Union {
    Foo(String),
    Bar(i32),
    #[doc = r" An unknown variant."]
    Unknown(Unknown),
}
impl ser::Serialize for Union {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map = s.serialize_map(Some(2))?;
        match self {
            Union::Foo(value) => {
                map.serialize_entry(&"type", &"foo")?;
                map.serialize_entry(&"foo", value)?;
            }
            Union::Bar(value) => {
                map.serialize_entry(&"type", &"bar")?;
                map.serialize_entry(&"bar", value)?;
            }
            Union::Unknown(value) => {
                map.serialize_entry(&"type", &value.type_)?;
                map.serialize_entry(&value.type_, &value.value)?;
            }
        }
        map.end()
    }
}
impl<'de> de::Deserialize<'de> for Union {
    fn deserialize<D>(d: D) -> Result<Union, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_map(Visitor_)
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = Union;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("union Union")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Union, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let v = match map.next_key::<UnionField_<Variant_>>()? {
            Some(UnionField_::Type) => {
                let variant = map.next_value()?;
                let key = map.next_key()?;
                match (variant, key) {
                    (Variant_::Foo, Some(Variant_::Foo)) => {
                        let value = map.next_value()?;
                        Union::Foo(value)
                    }
                    (Variant_::Bar, Some(Variant_::Bar)) => {
                        let value = map.next_value()?;
                        Union::Bar(value)
                    }
                    (Variant_::Unknown(type_), Some(Variant_::Unknown(b))) => {
                        if type_ == b {
                            let value = map.next_value()?;
                            Union::Unknown(Unknown { type_, value })
                        } else {
                            return Err(de::Error::invalid_value(de::Unexpected::Str(&type_), &&*b));
                        }
                    }
                    (variant, Some(key)) => {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Str(key.as_str()),
                            &variant.as_str(),
                        ));
                    }
                    (variant, None) => return Err(de::Error::missing_field(variant.as_str())),
                }
            }
            Some(UnionField_::Value(variant)) => {
                let value = match &variant {
                    Variant_::Foo => {
                        let value = map.next_value()?;
                        Union::Foo(value)
                    }
                    Variant_::Bar => {
                        let value = map.next_value()?;
                        Union::Bar(value)
                    }
                    Variant_::Unknown(type_) => {
                        let value = map.next_value()?;
                        Union::Unknown(Unknown {
                            type_: type_.clone(),
                            value,
                        })
                    }
                };
                if map.next_key::<UnionTypeField_>()?.is_some() {
                    let type_variant = map.next_value::<Variant_>()?;
                    if variant != type_variant {
                        return Err(de::Error::invalid_value(
                            de::Unexpected::Str(type_variant.as_str()),
                            &variant.as_str(),
                        ));
                    }
                }
                value
            }
            None => return Err(de::Error::missing_field("type")),
        };
        if map.next_key::<UnionField_<Variant_>>()?.is_some() {
            return Err(de::Error::invalid_length(3, &"type and value fields"));
        }
        Ok(v)
    }
}
#[derive(PartialEq)]
enum Variant_ {
    Foo,
    Bar,
    Unknown(Box<str>),
}
impl Variant_ {
    fn as_str(&self) -> &'static str {
        match self {
            Variant_::Foo => "foo",
            Variant_::Bar => "bar",
            Variant_::Unknown(_) => "unknown variant",
        }
    }
}
impl<'de> de::Deserialize<'de> for Variant_ {
    fn deserialize<D>(d: D) -> Result<Variant_, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        d.deserialize_str(VariantVisitor_)
    }
}
struct VariantVisitor_;
impl<'de> de::Visitor<'de> for VariantVisitor_ {
    type Value = Variant_;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("string")
    }
    fn visit_str<E>(self, value: &str) -> Result<Variant_, E>
    where
        E: de::Error,
    {
        let v = match value {
            "foo" => Variant_::Foo,
            "bar" => Variant_::Bar,
            value => Variant_::Unknown(value.to_string().into_boxed_str()),
        };
        Ok(v)
    }
}
#[doc = "An unknown variant of the Union union."]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unknown {
    type_: Box<str>,
    value: conjure_object::Value,
}
impl Unknown {
    #[doc = r" Returns the unknown variant's type name."]
    #[inline]
    pub fn type_(&self) -> &str {
        &self.type_
    }
}
