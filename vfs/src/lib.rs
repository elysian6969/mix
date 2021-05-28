#![feature(shrink_to)]
#![feature(toowned_clone_into)]

use std::borrow::{Borrow, Cow, ToOwned};
use std::ffi::{OsStr, OsString};
use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;
use tokio::fs;

pub use std::fs::{Metadata, Permissions};
pub use std::path::{Ancestors, StripPrefixError};
pub use tokio::fs::ReadDir;

#[repr(transparent)]
pub struct VfsPath {
    inner: Path,
}

#[derive(Clone)]
#[repr(transparent)]
pub struct VfsPathBuf {
    inner: PathBuf,
}

impl VfsPath {
    #[inline]
    pub fn new(s: &(impl AsRef<OsStr> + ?Sized)) -> &Self {
        unsafe { &*(s.as_ref() as *const OsStr as *const Self) }
    }

    #[inline]
    pub fn from_bytes<'a>(s: impl AsRef<[u8]> + 'a) -> &'a Self {
        unsafe { &*(s.as_ref() as *const [u8] as *const Self) }
    }

    pub fn ancestors(&self) -> Ancestors<'_> {
        self.inner.ancestors()
    }

    /// Yields the underlying [`OsStr`] slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use vfs::VfsPath;
    ///
    /// let s = VfsPath::new("foo.txt").as_os_str();
    ///
    /// assert_eq!(OsStr::new("foo.txt"), s);
    /// ```
    #[inline]
    pub fn as_os_str(&self) -> &OsStr {
        self.inner.as_os_str()
    }

    /// Yields a `&str` slice if the `VfaPath` is valid unicode.
    ///
    /// This conversion may entail doing a check for UTF-8 validity.
    /// Note that validation is performed because non-UTF-8 strings are
    /// perfectly valid for some OS.
    ///
    /// # Examples
    ///
    /// ```
    /// use vfs::VfsPath;
    ///
    /// let s = VfsPath::new("foo.txt").to_str();
    ///
    /// assert_eq!(Some("foo.txt"), s);
    /// ```
    #[inline]
    pub fn to_str(&self) -> Option<&str> {
        self.inner.to_str()
    }

    /// Converts a `VfsPath` to a [`Cow<str>`].
    ///
    /// Any non-Unicode sequences are replaced with
    /// [`U+FFFD REPLACEMENT CHARACTER`](std::char::REPLACEMENT_CHARACTER).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ffi::OsStr;
    /// use std::os::unix::ffi::OsStrExt;
    /// use vfs::VfsPath;
    ///
    /// let b = b"fo\xf0\x28\x8c\xbco.txt";
    /// let s = OsStr::from_bytes(b.as_slice());
    /// let p = VfsPath::new(s);
    ///
    /// assert_eq!(Some("fo�(��o.txt"), p.to_string_lossy());
    /// ```
    #[inline]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.inner.to_string_lossy()
    }

    /// Yields the underlying [`Path`] slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("foo.txt").as_std_path();
    ///
    /// assert_eq!(Path::new("foo.txt"), p);
    /// ```
    #[inline]
    pub fn as_std_path(&self) -> &Path {
        &self.inner
    }

    /// Converts this `VfsPath` to an owned `VfsPathBuf`.
    #[inline]
    pub fn to_path_buf(&self) -> VfsPathBuf {
        self.as_std_path().to_path_buf().into()
    }

    #[inline]
    pub fn into_path_buf(self: Box<VfsPath>) -> VfsPathBuf {
        let raw = Box::into_raw(self) as *mut Path;
        let inner = unsafe { Box::from_raw(raw) };
        let inner = PathBuf::from(inner);

        VfsPathBuf { inner }
    }

    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.inner.is_absolute()
    }

    #[inline]
    pub fn is_relative(&self) -> bool {
        self.inner.is_relative()
    }

    #[inline]
    pub fn has_root(&self) -> bool {
        self.inner.has_root()
    }

    #[inline]
    pub fn parent(&self) -> Option<&VfsPath> {
        self.inner.parent().map(VfsPath::new)
    }

    #[inline]
    pub fn extension(&self) -> Option<&OsStr> {
        self.inner.extension()
    }

    #[inline]
    pub fn extension_path(&self) -> Option<&VfsPath> {
        self.extension().map(VfsPath::new)
    }

    #[inline]
    pub fn extension_str(&self) -> Option<&str> {
        self.extension_path().and_then(VfsPath::to_str)
    }

    #[inline]
    pub fn file_name(&self) -> Option<&OsStr> {
        self.inner.file_name()
    }

    #[inline]
    pub fn file_name_path(&self) -> Option<&VfsPath> {
        self.file_name().map(VfsPath::new)
    }

    #[inline]
    pub fn file_name_str(&self) -> Option<&str> {
        self.file_name_path().and_then(VfsPath::to_str)
    }

    #[inline]
    pub fn file_stem(&self) -> Option<&OsStr> {
        self.inner.file_stem()
    }

    #[inline]
    pub fn file_stem_path(&self) -> Option<&VfsPath> {
        self.file_stem().map(VfsPath::new)
    }

    #[inline]
    pub fn file_stem_str(&self) -> Option<&str> {
        self.file_stem_path().and_then(VfsPath::to_str)
    }

    #[inline]
    pub fn strip_prefix(&self, base: impl AsRef<VfsPath>) -> Result<&VfsPath, StripPrefixError> {
        self.inner.strip_prefix(base.as_ref()).map(VfsPath::new)
    }

    #[inline]
    pub fn starts_with(&self, base: impl AsRef<VfsPath>) -> bool {
        self.inner.starts_with(base.as_ref())
    }

    #[inline]
    pub fn ends_with(&self, base: impl AsRef<VfsPath>) -> bool {
        self.inner.ends_with(base.as_ref())
    }

    #[inline]
    pub fn join(&self, path: impl AsRef<VfsPath>) -> VfsPathBuf {
        self.inner.join(path.as_ref()).into()
    }

    #[inline]
    pub fn with_file_name(&self, path: impl AsRef<OsStr>) -> VfsPathBuf {
        self.inner.with_file_name(path.as_ref()).into()
    }

    #[inline]
    pub fn with_extension(&self, path: impl AsRef<OsStr>) -> VfsPathBuf {
        self.inner.with_extension(path.as_ref()).into()
    }

    #[inline]
    pub async fn is_dir(&self) -> bool {
        self.try_is_dir().await.unwrap_or(false)
    }

    #[inline]
    pub async fn is_file(&self) -> bool {
        self.try_is_file().await.unwrap_or(false)
    }

    #[inline]
    pub async fn try_is_dir(&self) -> io::Result<bool> {
        self.metadata().await.map(|metadata| metadata.is_dir())
    }

    #[inline]
    pub async fn try_is_file(&self) -> io::Result<bool> {
        self.metadata().await.map(|metadata| metadata.is_file())
    }

    /// Returns the canonical, absolute form of a path with all intermediate
    /// components normalized and symbolic links resolved.
    ///
    /// This is an async method version of [`std::fs::canonicalize`](std::fs::canonicalize).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("../a/../foo.txt");
    ///
    /// // Assuming `a` is under `/`.
    /// assert_eq!(Ok(VfsPath::new("/a/foo.txt")), p.canonicalize().await);
    /// ```
    pub async fn canonicalize(&self) -> io::Result<VfsPathBuf> {
        fs::canonicalize(self).await.map(Into::into)
    }

    /// Creates a new, empty directory.
    ///
    /// This is an async method version of [`std::fs::create_dir`](std::fs::create_dir).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("some/dir");
    ///
    /// assert!(p.create_dir().await.is_ok());
    /// ```
    pub async fn create_dir(&self) -> io::Result<()> {
        fs::create_dir(self).await
    }

    /// Recursively creates a directory and all of its parent components if they
    /// are missing.
    ///
    /// This is an async method version of [`std::fs::create_dir_all`](std::fs::create_dir_all).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("some/dir");
    ///
    /// assert!(p.create_dir_all().await.is_ok());
    /// ```
    pub async fn create_dir_all(&self) -> io::Result<()> {
        fs::create_dir_all(self).await
    }

    /// Returns `true` if this path points at an existing entity.
    ///
    /// This is an async method version of [`std::Path::exists`](std::Path::exists).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("/does/not/exist");
    ///
    /// assert!(p.exists().await.is_err());
    /// ```
    pub async fn exists(&self) -> bool {
        self.metadata().await.is_ok()
    }

    pub async fn try_exists(&self) -> io::Result<bool> {
        match self.metadata().await {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    /// Creates a new hard link on the filesystem.
    ///
    /// This is an async method version of [`std::fs::hard_link`](std::fs::hard_link).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("/bin/a");
    ///
    /// assert!(p.hard_link("/bin/b").await.is_err());
    /// ```
    pub async fn hard_link(&self, to: impl AsRef<OsStr>) -> io::Result<()> {
        fs::hard_link(self, to.as_ref()).await
    }

    /// Queries the file system to get information about this file, directory, etc.
    ///
    /// This is an async method version of [`std::fs::metadata`](std::fs::metadata).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("/bin/a");
    ///
    /// if let Ok(metadata) = p.metadata().await {
    ///     // ...
    /// }
    /// ```
    pub async fn metadata(&self) -> io::Result<Metadata> {
        fs::metadata(self).await
    }

    /// Reads the entire contents of this file into a vector of bytes.
    ///
    /// This is an async method version of [`std::fs::read`](std::fs::read).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("/bin/a");
    ///
    /// if let Ok(slice) = p.read().await {
    ///     let string = String::from_utf8_lossy(&slice);
    ///
    ///     // ...
    /// }
    /// ```
    pub async fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(self).await
    }

    /// Returns a stream over the entries within a directory.
    ///
    /// This is an async method version of [`std::fs::read_dir`](std::fs::read_dir).
    pub async fn read_dir(&self) -> io::Result<ReadDir> {
        fs::read_dir(self).await
    }

    /// Reads a symbolic link, returning the path that the link points to.
    ///
    /// This is an async method version of [`std::fs::read_link`](std::fs::read_link).
    pub async fn read_link(&self) -> io::Result<VfsPathBuf> {
        fs::read_link(self).await.map(Into::into)
    }

    /// Reads the entire contents of this file into a String.
    ///
    /// This is an async method version of [`std::fs::read_to_string`](std::fs::read_to_string).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("/bin/a");
    ///
    /// if let Ok(string) = p.read_to_string().await {
    ///     // ...
    /// }
    /// ```
    pub async fn read_to_string(&self) -> io::Result<String> {
        fs::read_to_string(self).await
    }

    /// Removes an existing, empty directory.
    ///
    /// This is an async method version of [`std::fs::remove_dir`](std::fs::remove_dir).
    pub async fn remove_dir(&self) -> io::Result<()> {
        fs::remove_dir(self).await
    }

    /// Removes a directory, after removing all its contents.
    ///
    /// This is an async method version of [`std::fs::remove_dir_all`](std::fs::remove_dir_all).
    pub async fn remove_dir_all(&self) -> io::Result<()> {
        fs::remove_dir_all(self).await
    }

    /// Removes this path from the filesystem.
    ///
    /// This is an async method version of [`std::fs::remove_file`](std::fs::remove_file).
    pub async fn remove_file(&self) -> io::Result<()> {
        fs::remove_file(self).await
    }

    /// Renames this file or directory to a new name, replacing the original file if
    /// `to` already exists.
    ///
    /// This is an async method version of [`std::fs::rename`](std::fs::rename).
    pub async fn rename(&self, to: impl AsRef<OsStr>) -> io::Result<()> {
        fs::rename(self, to.as_ref()).await
    }

    /// Changes the permissions found on a file or a directory.
    ///
    /// This is an async method version of [`std::fs::set_permissions`](std::fs::set_permissions).
    pub async fn set_permissions(&self, permissions: Permissions) -> io::Result<()> {
        fs::set_permissions(self, permissions).await
    }

    /// Creates a new symbolic link on the filesystem.
    ///
    /// The `yo` path will be a symbolic link pointing to this path.
    ///
    /// This is an async method version of [`std::os::unix::fs::symlink`](std::os::unix::fs::symlink).
    pub async fn symlink(&self, to: impl AsRef<OsStr>) -> io::Result<()> {
        fs::symlink(self, to.as_ref()).await
    }

    /// Queries the file system metadata about this path.
    ///
    /// This is an async method version of [`std::fs::symlink_metadata`](std::fs::symlink_metadata).
    pub async fn symlink_metadata(&self) -> io::Result<Metadata> {
        fs::symlink_metadata(self).await
    }

    /// Writes the entrie slice of bytes to this file.
    ///
    /// This is an async method version of [`std::fs::write`](std::fs::write).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use vfs::VfsPath;
    ///
    /// let p = VfsPath::new("/bin/a");
    ///
    /// p.write(b"Hello, World!").await?;
    /// ```
    pub async fn write(&self, content: impl AsRef<[u8]>) -> io::Result<()> {
        fs::write(self, content.as_ref()).await
    }
}

impl VfsPathBuf {
    /// Allocates an empty `VfsPathBuf`.
    #[inline]
    pub fn new() -> Self {
        let inner = PathBuf::new();

        Self { inner }
    }

    /// Coerces to a `VfsPath` slice.
    #[inline]
    pub fn as_path(&self) -> &VfsPath {
        VfsPath::new(self)
    }

    /// Invokes `capacity` on the underlying instance of `PathBuf`.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Invokes `clear` on the underlying instance of `PathBuf`.
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// Consumes the `VfsPathBuf`, yielding its internal `OsString` storage.
    #[inline]
    pub fn into_os_string(self) -> OsString {
        self.into_path_buf().into_os_string()
    }

    /// Consumes the `VfsPathBuf`, yielding its internal `PathBuf` storage.
    #[inline]
    pub fn into_path_buf(self) -> PathBuf {
        self.inner
    }

    /// Converts this `VfsPathBuf` into a boxed `VfsPath`.
    #[inline]
    pub fn into_boxed_path(self) -> Box<VfsPath> {
        let boxed: Box<OsStr> = self.into_os_string().into_boxed_os_str();
        let raw = Box::into_raw(boxed) as *mut VfsPath;

        unsafe { Box::from_raw(raw) }
    }

    /// Truncates `self` to [`self.parent`].
    ///
    /// Returns `false` and does nothing if [`self.parent`] is [`None`].
    /// Otherwise, returns `true`.
    ///
    /// [`self.parent`]: VfsPath::parent
    ///
    /// # Examples
    ///
    /// ```
    /// use vfs::{VfsPath, VfsPathBuf};
    ///
    /// let mut p = VfsPathBuf::from("/spirited/away.rs");
    ///
    /// p.pop();
    /// assert_eq!(VfsPath::new("/spirited"), p);
    ///
    /// p.pop();
    /// assert_eq!(VfsPath::new("/"), p);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> bool {
        self.inner.pop()
    }

    #[inline]
    pub fn push(&mut self, path: impl AsRef<VfsPath>) {
        self.inner.push(path.as_ref())
    }

    /// Invokes `reserve` on the underlying instance of `PathBuf`.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// Invokes `reserve_exact` on the underlying instance of `PathBuf`.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.inner.reserve_exact(additional)
    }

    /// Updates [`self.extension`] to `extension`.
    ///
    /// Returns `false` and does nothing if [`self.file_name`] is [`None`],
    /// returns `true` and updates the extension otherwise.
    ///
    /// If [`self.extension`] is [`None`], the extension is added; otherwise
    /// it is replaced.
    ///
    /// [`self.file_name`]: VfsPath::file_name
    /// [`self.extension`]: VfsPath::extension
    ///
    /// # Examples
    ///
    /// ```
    /// use vfs::{VfsPath, VfsPathBuf};
    ///
    /// let mut p = VfsPathBuf::from("/feel/the");
    ///
    /// p.set_extension("force");
    /// assert_eq!(VfsPath::new("/feel/the.force"), p);
    ///
    /// p.set_extension("dark_side");
    /// assert_eq!(VfsPath::new("/feel/the.dark_side"), p);
    /// ```
    #[inline]
    pub fn set_extension(&mut self, extension: impl AsRef<OsStr>) -> bool {
        self.inner.set_extension(extension.as_ref())
    }

    /// Updates [`self.file_name`] to `file_name`.
    ///
    /// If [`self.file_name`] was [`None`], this is equivalent to pushing
    /// `file_name`.
    ///
    /// Otherwise it is equivalent to calling [`pop`] and then pushing
    /// `file_name`. The new path will be a sibling of the original path.
    /// (That is, it will have the same parent.)
    ///
    /// [`self.file_name`]: VfsPath::file_name
    /// [`pop`]: VfsPathBuf::pop
    ///
    /// # Examples
    ///
    /// ```
    /// use vfs::{VfsPath, VfsPathBuf};
    ///
    /// let mut p = VfsPathBuf::from("/");
    ///
    /// assert_eq!(p.file_name(), None);
    ///
    /// p.set_file_name("bar");
    /// assert_eq!(VfsPath::from("/bar"), p);
    /// assert!(p.file_name().is_some());
    ///
    /// p.set_file_name("baz.txt");
    /// assert!(VfsPath::from("/baz.txt"), p);
    /// ```
    #[inline]
    pub fn set_file_name(&mut self, file_name: impl AsRef<OsStr>) {
        self.inner.set_file_name(file_name.as_ref())
    }

    /// Invokes `shrink_to` on the underlying instance of `PathBuf`.
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity)
    }

    /// Invokes `shrink_to_fit` on the underlying instance of `PathBuf`.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit()
    }
}

// AsRef impls

impl AsRef<OsStr> for VfsPath {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl AsRef<Path> for VfsPath {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_std_path()
    }
}

impl AsRef<OsStr> for VfsPathBuf {
    #[inline]
    fn as_ref(&self) -> &OsStr {
        self.as_path().as_os_str()
    }
}

impl AsRef<Path> for VfsPathBuf {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.as_path().as_std_path()
    }
}

impl AsRef<VfsPath> for VfsPathBuf {
    #[inline]
    fn as_ref(&self) -> &VfsPath {
        self.as_path()
    }
}

// Deref impls

impl Deref for VfsPathBuf {
    type Target = VfsPath;

    #[inline]
    fn deref(&self) -> &Self::Target {
        VfsPath::new(&self.inner)
    }
}

// Borrow impls

impl Borrow<VfsPath> for VfsPathBuf {
    #[inline]
    fn borrow(&self) -> &VfsPath {
        self.deref()
    }
}

// Default impls

impl Default for VfsPathBuf {
    #[inline]
    fn default() -> VfsPathBuf {
        VfsPathBuf::new()
    }
}

// Cow impls

impl<'a> From<&'a VfsPath> for Cow<'a, VfsPath> {
    #[inline]
    fn from(p: &'a VfsPath) -> Cow<'a, VfsPath> {
        Cow::Borrowed(p)
    }
}

impl<'a> From<VfsPathBuf> for Cow<'a, VfsPath> {
    #[inline]
    fn from(p: VfsPathBuf) -> Cow<'a, VfsPath> {
        Cow::Owned(p)
    }
}

impl<'a> From<&'a VfsPathBuf> for Cow<'a, VfsPath> {
    #[inline]
    fn from(p: &'a VfsPathBuf) -> Cow<'a, VfsPath> {
        Cow::Borrowed(p.as_path())
    }
}

impl<'a> From<Cow<'a, VfsPath>> for VfsPathBuf {
    #[inline]
    fn from(p: Cow<'a, VfsPath>) -> VfsPathBuf {
        p.into_owned()
    }
}

impl From<Cow<'_, VfsPath>> for Box<VfsPath> {
    #[inline]
    fn from(cow: Cow<'_, VfsPath>) -> Box<VfsPath> {
        match cow {
            Cow::Borrowed(path) => Box::from(path),
            Cow::Owned(path) => Box::from(path),
        }
    }
}

// From impls

impl<S: AsRef<OsStr> + ?Sized> From<&S> for VfsPathBuf {
    #[inline]
    fn from(s: &S) -> Self {
        VfsPathBuf::from(s.as_ref().to_os_string())
    }
}

impl From<OsString> for VfsPathBuf {
    /// Converts a `OsString` into a `VfsPathBuf`.
    ///
    /// This conversion does not allocate or copy memory.
    #[inline]
    fn from(s: OsString) -> Self {
        Self { inner: s.into() }
    }
}

impl From<PathBuf> for VfsPathBuf {
    /// Converts a `PathBuf` into a `VfsPathBuf`.
    ///
    /// This conversion does not allocate or copy memory.
    #[inline]
    fn from(p: PathBuf) -> Self {
        Self { inner: p }
    }
}

impl From<String> for VfsPathBuf {
    /// Converts a `String` into a `VfsPathBuf`.
    ///
    /// This conversion does not allocate or copy memory.
    #[inline]
    fn from(s: String) -> Self {
        Self { inner: s.into() }
    }
}

// Boxed impls

impl From<Box<VfsPath>> for VfsPathBuf {
    /// Converts a `Box<VfsPath>` into a `VfsPathBuf`.
    ///
    /// This conversion does not allocate or copy memory.
    #[inline]
    fn from(b: Box<VfsPath>) -> VfsPathBuf {
        b.into_path_buf()
    }
}

impl From<&VfsPath> for Box<VfsPath> {
    fn from(path: &VfsPath) -> Box<VfsPath> {
        let boxed: Box<Path> = path.as_std_path().into();
        let raw = Box::into_raw(boxed) as *mut VfsPath;
        unsafe { Box::from_raw(raw) }
    }
}

impl From<VfsPathBuf> for Box<VfsPath> {
    /// Converts a `VfsPathBuf` into a `Box<VfsPath>`.
    ///
    /// This conversion currently should not allocate memory, but this behavior is not guaranteed on all platforms or in all future versions.
    #[inline]
    fn from(p: VfsPathBuf) -> Box<VfsPath> {
        p.into_boxed_path()
    }
}

// Rc impls

impl From<VfsPathBuf> for Arc<VfsPath> {
    /// Converts a `VfsPathBuf` into an `Arc` by moving the `VfsPathBuf` data into a new `Arc` buffer.
    #[inline]
    fn from(p: VfsPathBuf) -> Arc<VfsPath> {
        let arc: Arc<OsStr> = Arc::from(p.into_os_string());

        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const VfsPath) }
    }
}

impl From<&VfsPath> for Arc<VfsPath> {
    /// Converts a `VfsPath` into an `Arc` by copying the `VfsPath` data into a new `Arc` buffer.
    #[inline]
    fn from(p: &VfsPath) -> Arc<VfsPath> {
        let arc: Arc<OsStr> = Arc::from(p.as_os_str());

        unsafe { Arc::from_raw(Arc::into_raw(arc) as *const VfsPath) }
    }
}

impl From<VfsPathBuf> for Rc<VfsPath> {
    /// Converts a `VfsPathBuf` into an `Rd` by moving the `VfsPathBuf` data into a new `Rc` buffer.
    #[inline]
    fn from(p: VfsPathBuf) -> Rc<VfsPath> {
        let rc: Rc<OsStr> = Rc::from(p.into_os_string());

        unsafe { Rc::from_raw(Rc::into_raw(rc) as *const VfsPath) }
    }
}

impl From<&VfsPath> for Rc<VfsPath> {
    /// Converts a `VfsPath` into an `Rc` by copying the `VfsPath` data into a new `Rc` buffer.
    #[inline]
    fn from(p: &VfsPath) -> Rc<VfsPath> {
        let arc: Rc<OsStr> = Rc::from(p.as_os_str());

        unsafe { Rc::from_raw(Rc::into_raw(arc) as *const VfsPath) }
    }
}

// ToOwned impls

impl ToOwned for VfsPath {
    type Owned = VfsPathBuf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        self.to_path_buf()
    }

    #[inline]
    fn clone_into(&self, target: &mut Self::Owned) {
        self.inner.clone_into(&mut target.inner);
    }
}
