use cargo_metadata::semver::Version as OriginalVersion;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SbuildVersion {
    inner: OriginalVersion,
    original_string: Cow<'static, str>,
}

impl fmt::Display for SbuildVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original_string)
    }
}

impl SbuildVersion {
    pub fn as_str(&self) -> &str {
        &self.original_string
    }

    // Extract version from the first line of a string
    fn extract_version(s: &str) -> Option<String> {
        let first_line = s.lines().next()?;

        if let Some(start_idx) = first_line.find("sbuild (Debian sbuild) ") {
            let start_pos = start_idx + "sbuild (Debian sbuild) ".len();
            if let Some(end_idx) = first_line[start_pos..].find(' ') {
                return Some(first_line[start_pos..(start_pos + end_idx)].to_string());
            } else {
                return Some(first_line[start_pos..].trim().to_string());
            }
        }
        None
    }
}

impl Serialize for SbuildVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.original_string)
    }
}

impl Deref for SbuildVersion {
    type Target = OriginalVersion;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for SbuildVersion {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<OriginalVersion> for SbuildVersion {
    fn from(version: OriginalVersion) -> Self {
        let original_string = Cow::Owned(version.to_string());
        SbuildVersion {
            inner: version,
            original_string,
        }
    }
}

impl<'a> TryFrom<&'a str> for SbuildVersion {
    type Error = cargo_metadata::semver::Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let version_str = if s.contains('\n') {
            SbuildVersion::extract_version(s)
                .unwrap_or_else(|| s.lines().next().unwrap_or(s).to_string())
        } else {
            s.to_string()
        };

        let inner = OriginalVersion::parse(&version_str)?;
        Ok(SbuildVersion {
            inner,
            original_string: Cow::Owned(version_str),
        })
    }
}

impl TryFrom<String> for SbuildVersion {
    type Error = cargo_metadata::semver::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        <SbuildVersion as TryFrom<&str>>::try_from(&s)
    }
}

impl<'de> Deserialize<'de> for SbuildVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SbuildVersionVisitor;
        impl<'de> de::Visitor<'de> for SbuildVersionVisitor {
            type Value = SbuildVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string containing a valid version, possibly in a multiline string",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let version_str = if value.contains('\n') {
                    SbuildVersion::extract_version(value).ok_or_else(|| {
                        de::Error::custom("Could not extract version from the first line")
                    })?
                } else {
                    value.to_string()
                };

                let inner = OriginalVersion::parse(&version_str).map_err(de::Error::custom)?;
                Ok(SbuildVersion {
                    inner,
                    original_string: Cow::Owned(version_str),
                })
            }
        }

        deserializer.deserialize_string(SbuildVersionVisitor)
    }
}
