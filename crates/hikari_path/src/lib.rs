use std::ops::Deref;

pub const ENGINE_URI: &str = "engine://";
pub const USER_URI: &str = "user://";
pub const SEPARATOR: &str = "/";

pub struct AssetPath {
    inner: str,
}
impl AssetPath {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &AssetPath {
        unsafe { &*(s.as_ref() as *const str as *const AssetPath) }
    }
}
pub struct AssetPathBuf {
    inner: String,
}

impl AssetPathBuf {
    pub fn empty() -> Self {
        Self {
            inner: String::new()
        }
    }
    pub fn engine() -> Self {
        Self {
            inner: String::from(ENGINE_URI),
        }
    }
    pub fn user() -> Self {
        Self {
            inner: String::from(USER_URI)
        }
    }
    pub fn as_asset_path(&self) -> &AssetPath {
        self
    }
    pub fn push<P: AsRef<AssetPath>>(&mut self, path: P) {
        self.inner.push_str(&path.as_ref().inner);
        self.inner.push_str(SEPARATOR);
    }
}

impl Deref for AssetPathBuf {
    type Target = AssetPath;

    fn deref(&self) -> &Self::Target {
        AssetPath::new(&self.inner)
    }
}