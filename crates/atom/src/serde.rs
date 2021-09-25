use crate::{Atom, AtomReq};
use core::fmt;
use serde::de::{Deserialize, Deserializer, Error, Visitor};
use serde::ser::{Serialize, Serializer};

impl Serialize for Atom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl Serialize for AtomReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Atom {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AtomVisitor;

        impl<'de> Visitor<'de> for AtomVisitor {
            type Value = Atom;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("atom")
            }

            fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                string.parse().map_err(Error::custom)
            }
        }

        deserializer.deserialize_str(AtomVisitor)
    }
}

impl<'de> Deserialize<'de> for AtomReq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AtomReqVisitor;

        impl<'de> Visitor<'de> for AtomReqVisitor {
            type Value = AtomReq;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("atom requirement")
            }

            fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                string.parse().map_err(Error::custom)
            }
        }

        deserializer.deserialize_str(AtomReqVisitor)
    }
}
