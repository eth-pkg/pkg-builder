use serde::{de, Deserialize, Deserializer, Serialize};
use std::fmt;
use std::ops::{Deref, DerefMut};
use url::Url as OriginalUrl;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url(OriginalUrl);

impl Serialize for Url {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.as_str())
    }
}

impl Deref for Url {
    type Target = OriginalUrl;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Url {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<OriginalUrl> for Url {
    fn from(url: OriginalUrl) -> Self {
        Url(url)
    }
}

impl Url {
    pub fn into_inner(self) -> OriginalUrl {
        self.0
    }
}


impl<'de> Deserialize<'de> for Url {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UrlVisitor;

        impl<'de> de::Visitor<'de> for UrlVisitor {
            type Value = Url;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string containing a valid URL")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                OriginalUrl::parse(value)
                    .map(Url)
                    .map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_string(UrlVisitor)
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
