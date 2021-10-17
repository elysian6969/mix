pub use crate::error::Error;
use mix_id::{PackageId, RepositoryId};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::convert::TryInto;
use std::str::FromStr;
use std::{cmp, fmt};

mod error;

#[cfg(feature = "serde")]
mod serde;

mod atom;
mod requirement;

pub use crate::atom::Atom;
pub use crate::requirement::Requirement;
