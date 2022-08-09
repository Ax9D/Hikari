use serde::{Deserialize, Serialize};

use crate::{manager::LoadContext, meta::MetaData};

pub trait Asset: Send + Sync + Load + 'static {
    const NAME: &'static str;
    fn extensions<'a>() -> &'a [&'static str];
}

pub trait Load {
    type Loader: Send + Sync + 'static;
    type LoadSettings: for<'a> Deserialize<'a> + Serialize + Send + Sync + Default + Clone + 'static;

    fn load(
        loader: &Self::Loader,
        data: &[u8],
        meta: &MetaData<Self>,
        context: &mut LoadContext,
    ) -> Result<Self, crate::Error>
    where
        Self: Sized;
}
