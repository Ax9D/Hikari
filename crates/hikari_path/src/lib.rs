pub struct AssetPath {
    inner: str,
}
pub struct AssetPathBuf {
    inner: String,
}

impl AssetPathBuf {
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }
}
