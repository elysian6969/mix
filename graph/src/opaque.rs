use std::sync::Arc;

/// Opaque group identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Group {
    inner: Arc<String>,
}

/// Opaque package identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Package {
    inner: Arc<String>,
}

/// Opaque package identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Relation {
    package: Package,
    inner: Arc<String>,
}

impl Group {
    /// Coerces to a [`str`] slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl Package {
    /// Coerces to a [`str`] slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }

    pub fn relation(&self, relation: impl AsRef<str> + ?Sized) -> Relation {
        Relation {
            package: self.clone(),
            relation: Arc::new(relation.as_ref().to_string()),
        }
    }
}

// AsRef Impls

impl AsRef<str> for Group {
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for Package {
    #[inline]
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

// From AsRef Impls

impl<S: AsRef<str> + ?Sized> From<&S> for Group {
    #[inline]
    fn from(s: &S) -> Self {
        Self::from(s.to_string())
    }
}

impl<S: AsRef<str> + ?Sized> From<&S> for Package {
    #[inline]
    fn from(s: &S) -> Self {
        Self::from(s.to_string())
    }
}

// From String impls

impl From<String> for Group {
    /// Converts a `String` into a `Group`.
    #[inline]
    fn from(string: String) -> Self {
        Self::from(Arc::new(string))
    }
}

impl From<String> for Package {
    /// Converts a `String` into a `Package`.
    #[inline]
    fn from(string: String) -> Self {
        Self::from(Arc::new(string))
    }
}

// From Arc impls

impl From<Arc<String>> for Group {
    /// Converts a `String` into a `Group`.
    ///
    /// This conversion does not allocate or copy memory.
    #[inline]
    fn from(arc: Arc<String>) -> Self {
        Self { inner: arc }
    }
}

impl From<Arc<String>> for Group {
    /// Converts a `String` into a `Group`.
    ///
    /// This conversion does not allocate or copy memory.
    #[inline]
    fn from(arc: Arc<String>) -> Self {
        Self { inner: arc }
    }
}
