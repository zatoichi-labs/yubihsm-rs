//! Logical partitions within the `YubiHSM2`, allowing several applications to
//! share the device concurrently

use std::fmt;

use failure::Error;
use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer, Visitor};

use super::SessionError;

/// Logical partition within the `YubiHSM2`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Domain(pub(crate) u8);

impl Domain {
    /// Create a new Domain
    pub fn new(domain: u8) -> Result<Self, Error> {
        if domain < 1 || domain > 16 {
            fail!(SessionError::ProtocolError, "invalid domain: {}", domain);
        }

        Ok(Domain(domain))
    }

    /// Create a Domain from a byte serialization
    #[inline]
    pub fn from_u8(domain: u8) -> Result<Self, Error> {
        Self::new(domain)
    }

    /// Serialize this domain as a byte
    pub fn to_u8(&self) -> u8 {
        self.0
    }
}

/// A collection of Domains
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Domains(Vec<Domain>);

impl Domains {
    /// Decode a u16 of domain bitflags into a Domains value
    pub fn from_u16(bitfield: u16) -> Self {
        let mut result = vec![];

        for i in 1..16 {
            if bitfield & (1 << i) != 0 {
                result.push(Domain::new(i).unwrap())
            }
        }

        Domains(result)
    }

    /// Convert an array of Domain objects to a 16-bit integer bitfield
    pub fn to_u16(&self) -> u16 {
        self.0
            .iter()
            .fold(0, |result, domain| result | (1 << domain.0))
    }
}

impl<'a> From<&'a [Domain]> for Domains {
    /// Create a domains object from a slice
    fn from(domains: &'a [Domain]) -> Self {
        Domains(domains.into())
    }
}

impl Serialize for Domains {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(self.to_u16())
    }
}

impl<'de> Deserialize<'de> for Domains {
    fn deserialize<D>(deserializer: D) -> Result<Domains, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DomainsVisitor;

        impl<'de> Visitor<'de> for DomainsVisitor {
            type Value = Domains;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("2-bytes containing domain bitflags")
            }

            fn visit_u16<E>(self, value: u16) -> Result<Domains, E>
            where
                E: de::Error,
            {
                Ok(Domains::from_u16(value))
            }
        }

        deserializer.deserialize_u16(DomainsVisitor)
    }
}
