use semver::Version as OriginalVersion;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Version {
    inner: OriginalVersion,
    original_string: Cow<'static, str>,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original_string)
    }
}

impl Version {
    pub fn as_str(&self) -> &str {
        &self.original_string
    }
}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.original_string)
    }
}

impl Deref for Version {
    type Target = OriginalVersion;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Version {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<OriginalVersion> for Version {
    fn from(version: OriginalVersion) -> Self {
        let original_string = Cow::Owned(version.to_string());
        Version {
            inner: version,
            original_string,
        }
    }
}

impl Version {
    pub fn into_inner(self) -> OriginalVersion {
        self.inner
    }
}

impl<'a> TryFrom<&'a str> for Version {
    type Error = semver::Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let inner = OriginalVersion::parse(s)?;
        Ok(Version {
            inner,
            original_string: Cow::Owned(s.to_string()),
        })
    }
}

impl TryFrom<String> for Version {
    type Error = semver::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        <Version as TryFrom<&str>>::try_from(&s)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'de> de::Visitor<'de> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string containing a valid semantic version (e.g., 1.2.3)")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let inner = OriginalVersion::parse(value).map_err(de::Error::custom)?;
                Ok(Version {
                    inner,
                    original_string: Cow::Owned(value.to_string()),
                })
            }
        }

        deserializer.deserialize_string(VersionVisitor)
    }
}
