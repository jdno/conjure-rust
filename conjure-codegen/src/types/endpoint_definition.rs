use conjure::serde::ser::SerializeMap as SerializeMap_;
use conjure::serde::{de, ser};
use std::fmt;
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EndpointDefinition {
    endpoint_name: super::EndpointName,
    http_method: super::HttpMethod,
    http_path: super::HttpPath,
    auth: Option<Box<super::AuthType>>,
    args: Vec<super::ArgumentDefinition>,
    returns: Option<Box<super::Type>>,
    docs: Option<super::Documentation>,
    deprecated: Option<super::Documentation>,
    markers: Vec<super::Type>,
}
impl EndpointDefinition {
    #[doc = r" Returns a new builder."]
    #[inline]
    pub fn builder() -> Builder {
        Default::default()
    }
    #[inline]
    pub fn endpoint_name(&self) -> &super::EndpointName {
        &self.endpoint_name
    }
    #[inline]
    pub fn http_method(&self) -> &super::HttpMethod {
        &self.http_method
    }
    #[inline]
    pub fn http_path(&self) -> &super::HttpPath {
        &self.http_path
    }
    #[inline]
    pub fn auth(&self) -> Option<&super::AuthType> {
        self.auth.as_ref().map(|o| &**o)
    }
    #[inline]
    pub fn args(&self) -> &[super::ArgumentDefinition] {
        &*self.args
    }
    #[inline]
    pub fn returns(&self) -> Option<&super::Type> {
        self.returns.as_ref().map(|o| &**o)
    }
    #[inline]
    pub fn docs(&self) -> Option<&super::Documentation> {
        self.docs.as_ref().map(|o| &*o)
    }
    #[inline]
    pub fn deprecated(&self) -> Option<&super::Documentation> {
        self.deprecated.as_ref().map(|o| &*o)
    }
    #[inline]
    pub fn markers(&self) -> &[super::Type] {
        &*self.markers
    }
}
#[derive(Debug, Clone, Default)]
pub struct Builder {
    endpoint_name: Option<super::EndpointName>,
    http_method: Option<super::HttpMethod>,
    http_path: Option<super::HttpPath>,
    auth: Option<Box<super::AuthType>>,
    args: Vec<super::ArgumentDefinition>,
    returns: Option<Box<super::Type>>,
    docs: Option<super::Documentation>,
    deprecated: Option<super::Documentation>,
    markers: Vec<super::Type>,
}
impl Builder {
    #[inline]
    pub fn endpoint_name(&mut self, endpoint_name: super::EndpointName) -> &mut Self {
        self.endpoint_name = Some(endpoint_name);
        self
    }
    #[inline]
    pub fn http_method(&mut self, http_method: super::HttpMethod) -> &mut Self {
        self.http_method = Some(http_method);
        self
    }
    #[inline]
    pub fn http_path(&mut self, http_path: super::HttpPath) -> &mut Self {
        self.http_path = Some(http_path);
        self
    }
    pub fn auth<T>(&mut self, auth: T) -> &mut Self
    where
        T: Into<Option<super::AuthType>>,
    {
        self.auth = auth.into().map(Box::new);
        self
    }
    pub fn args<T>(&mut self, args: T) -> &mut Self
    where
        T: IntoIterator<Item = super::ArgumentDefinition>,
    {
        self.args = args.into_iter().collect();
        self
    }
    pub fn extend_args<T>(&mut self, args: T) -> &mut Self
    where
        T: IntoIterator<Item = super::ArgumentDefinition>,
    {
        self.args.extend(args);
        self
    }
    pub fn returns<T>(&mut self, returns: T) -> &mut Self
    where
        T: Into<Option<super::Type>>,
    {
        self.returns = returns.into().map(Box::new);
        self
    }
    pub fn docs<T>(&mut self, docs: T) -> &mut Self
    where
        T: Into<Option<super::Documentation>>,
    {
        self.docs = docs.into();
        self
    }
    pub fn deprecated<T>(&mut self, deprecated: T) -> &mut Self
    where
        T: Into<Option<super::Documentation>>,
    {
        self.deprecated = deprecated.into();
        self
    }
    pub fn markers<T>(&mut self, markers: T) -> &mut Self
    where
        T: IntoIterator<Item = super::Type>,
    {
        self.markers = markers.into_iter().collect();
        self
    }
    pub fn extend_markers<T>(&mut self, markers: T) -> &mut Self
    where
        T: IntoIterator<Item = super::Type>,
    {
        self.markers.extend(markers);
        self
    }
    #[doc = r" Constructs a new instance of the type."]
    #[doc = r""]
    #[doc = r" # Panics"]
    #[doc = r""]
    #[doc = r" Panics if a required field was not set."]
    #[inline]
    pub fn build(&self) -> EndpointDefinition {
        EndpointDefinition {
            endpoint_name: self
                .endpoint_name
                .clone()
                .expect("field endpoint_name was not set"),
            http_method: self
                .http_method
                .clone()
                .expect("field http_method was not set"),
            http_path: self.http_path.clone().expect("field http_path was not set"),
            auth: self.auth.clone(),
            args: self.args.clone(),
            returns: self.returns.clone(),
            docs: self.docs.clone(),
            deprecated: self.deprecated.clone(),
            markers: self.markers.clone(),
        }
    }
}
impl From<EndpointDefinition> for Builder {
    #[inline]
    fn from(_v: EndpointDefinition) -> Builder {
        Builder {
            endpoint_name: Some(_v.endpoint_name),
            http_method: Some(_v.http_method),
            http_path: Some(_v.http_path),
            auth: _v.auth,
            args: _v.args,
            returns: _v.returns,
            docs: _v.docs,
            deprecated: _v.deprecated,
            markers: _v.markers,
        }
    }
}
impl ser::Serialize for EndpointDefinition {
    fn serialize<S_>(&self, s: S_) -> Result<S_::Ok, S_::Error>
    where
        S_: ser::Serializer,
    {
        let mut size = 3usize;
        let skip_auth = self.auth.is_none();
        if !skip_auth {
            size += 1;
        }
        let skip_args = self.args.is_empty();
        if !skip_args {
            size += 1;
        }
        let skip_returns = self.returns.is_none();
        if !skip_returns {
            size += 1;
        }
        let skip_docs = self.docs.is_none();
        if !skip_docs {
            size += 1;
        }
        let skip_deprecated = self.deprecated.is_none();
        if !skip_deprecated {
            size += 1;
        }
        let skip_markers = self.markers.is_empty();
        if !skip_markers {
            size += 1;
        }
        let mut map = s.serialize_map(Some(size))?;
        map.serialize_entry(&"endpointName", &self.endpoint_name)?;
        map.serialize_entry(&"httpMethod", &self.http_method)?;
        map.serialize_entry(&"httpPath", &self.http_path)?;
        if !skip_auth {
            map.serialize_entry(&"auth", &self.auth)?;
        }
        if !skip_args {
            map.serialize_entry(&"args", &self.args)?;
        }
        if !skip_returns {
            map.serialize_entry(&"returns", &self.returns)?;
        }
        if !skip_docs {
            map.serialize_entry(&"docs", &self.docs)?;
        }
        if !skip_deprecated {
            map.serialize_entry(&"deprecated", &self.deprecated)?;
        }
        if !skip_markers {
            map.serialize_entry(&"markers", &self.markers)?;
        }
        map.end()
    }
}
impl<'de> de::Deserialize<'de> for EndpointDefinition {
    fn deserialize<D_>(d: D_) -> Result<EndpointDefinition, D_::Error>
    where
        D_: de::Deserializer<'de>,
    {
        d.deserialize_struct(
            "EndpointDefinition",
            &[
                "endpointName",
                "httpMethod",
                "httpPath",
                "auth",
                "args",
                "returns",
                "docs",
                "deprecated",
                "markers",
            ],
            Visitor_,
        )
    }
}
struct Visitor_;
impl<'de> de::Visitor<'de> for Visitor_ {
    type Value = EndpointDefinition;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("map")
    }
    fn visit_map<A_>(self, mut map_: A_) -> Result<EndpointDefinition, A_::Error>
    where
        A_: de::MapAccess<'de>,
    {
        let mut endpoint_name = None;
        let mut http_method = None;
        let mut http_path = None;
        let mut auth = None;
        let mut args = None;
        let mut returns = None;
        let mut docs = None;
        let mut deprecated = None;
        let mut markers = None;
        while let Some(field_) = map_.next_key()? {
            match field_ {
                Field_::EndpointName => endpoint_name = Some(map_.next_value()?),
                Field_::HttpMethod => http_method = Some(map_.next_value()?),
                Field_::HttpPath => http_path = Some(map_.next_value()?),
                Field_::Auth => auth = Some(map_.next_value()?),
                Field_::Args => args = Some(map_.next_value()?),
                Field_::Returns => returns = Some(map_.next_value()?),
                Field_::Docs => docs = Some(map_.next_value()?),
                Field_::Deprecated => deprecated = Some(map_.next_value()?),
                Field_::Markers => markers = Some(map_.next_value()?),
                Field_::Unknown_ => {
                    map_.next_value::<de::IgnoredAny>()?;
                }
            }
        }
        let endpoint_name = match endpoint_name {
            Some(v) => v,
            None => return Err(de::Error::missing_field("endpointName")),
        };
        let http_method = match http_method {
            Some(v) => v,
            None => return Err(de::Error::missing_field("httpMethod")),
        };
        let http_path = match http_path {
            Some(v) => v,
            None => return Err(de::Error::missing_field("httpPath")),
        };
        let auth = match auth {
            Some(v) => v,
            None => Default::default(),
        };
        let args = match args {
            Some(v) => v,
            None => Default::default(),
        };
        let returns = match returns {
            Some(v) => v,
            None => Default::default(),
        };
        let docs = match docs {
            Some(v) => v,
            None => Default::default(),
        };
        let deprecated = match deprecated {
            Some(v) => v,
            None => Default::default(),
        };
        let markers = match markers {
            Some(v) => v,
            None => Default::default(),
        };
        Ok(EndpointDefinition {
            endpoint_name,
            http_method,
            http_path,
            auth,
            args,
            returns,
            docs,
            deprecated,
            markers,
        })
    }
}
enum Field_ {
    EndpointName,
    HttpMethod,
    HttpPath,
    Auth,
    Args,
    Returns,
    Docs,
    Deprecated,
    Markers,
    Unknown_,
}
impl<'de> de::Deserialize<'de> for Field_ {
    fn deserialize<D_>(d: D_) -> Result<Field_, D_::Error>
    where
        D_: de::Deserializer<'de>,
    {
        d.deserialize_str(FieldVisitor_)
    }
}
struct FieldVisitor_;
impl<'de> de::Visitor<'de> for FieldVisitor_ {
    type Value = Field_;
    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("string")
    }
    fn visit_str<E_>(self, value: &str) -> Result<Field_, E_>
    where
        E_: de::Error,
    {
        let v = match value {
            "endpointName" => Field_::EndpointName,
            "httpMethod" => Field_::HttpMethod,
            "httpPath" => Field_::HttpPath,
            "auth" => Field_::Auth,
            "args" => Field_::Args,
            "returns" => Field_::Returns,
            "docs" => Field_::Docs,
            "deprecated" => Field_::Deprecated,
            "markers" => Field_::Markers,
            _ => Field_::Unknown_,
        };
        Ok(v)
    }
}
