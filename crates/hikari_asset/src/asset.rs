use serde::{Deserialize, Serialize};

pub trait Asset: 'static + Send + Sync {
    type Settings: Send + Sync + Default + Clone + Serialize + for<'a> Deserialize<'a>;
}
