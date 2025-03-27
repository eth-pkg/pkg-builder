use cargo_metadata::semver::Version as OriginalVersion;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AutopkgVersion {
    inner: OriginalVersion,
    original_string: Cow<'static, str>,
}

impl fmt::Display for AutopkgVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original_string)
    }
}

impl AutopkgVersion {
    pub fn as_str(&self) -> &str {
        &self.original_string
    }

    // Helper to normalize version strings like "2.5" to "0.2.5"
    fn normalize_version(s: &str) -> String {
        if s.matches('.').count() == 1 {
            format!("0.{}", s)
        } else {
            s.to_string()
        }
    }

    // Helper to denormalize version strings like "0.2.5" back to "2.5"
    fn denormalize_version(s: &str) -> String {
        if s.starts_with("0.") {
            s[2..].to_string()
        } else {
            s.to_string()
        }
    }
}

impl Serialize for AutopkgVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Remove the leading "0." if it was added during deserialization
        let output = AutopkgVersion::denormalize_version(&self.original_string);
        serializer.serialize_str(&output)
    }
}

impl Deref for AutopkgVersion {
    type Target = OriginalVersion;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AutopkgVersion {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<OriginalVersion> for AutopkgVersion {
    fn from(version: OriginalVersion) -> Self {
        let original_string = Cow::Owned(version.to_string());
        AutopkgVersion {
            inner: version,
            original_string,
        }
    }
}

impl<'a> TryFrom<&'a str> for AutopkgVersion {
    type Error = cargo_metadata::semver::Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let normalized = AutopkgVersion::normalize_version(s);
        let inner = OriginalVersion::parse(&normalized)?;
        Ok(AutopkgVersion {
            inner,
            original_string: Cow::Owned(s.to_string()), // Keep the original format
        })
    }
}

impl TryFrom<String> for AutopkgVersion {
    type Error = cargo_metadata::semver::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        <AutopkgVersion as TryFrom<&str>>::try_from(&s)
    }
}

impl<'de> Deserialize<'de> for AutopkgVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AutopkgVersionVisitor;
        impl<'de> de::Visitor<'de> for AutopkgVersionVisitor {
            type Value = AutopkgVersion;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string containing a valid version (e.g., 2.5 or 1.2.3)")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let normalized = AutopkgVersion::normalize_version(value);
                let inner = OriginalVersion::parse(&normalized).map_err(de::Error::custom)?;
                Ok(AutopkgVersion {
                    inner,
                    original_string: Cow::Owned(value.to_string()), // Keep original format
                })
            }
        }
        deserializer.deserialize_string(AutopkgVersionVisitor)
    }
}
