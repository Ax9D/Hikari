
pub trait State: Sync + Send + 'static {}

impl<T> State for T where T: Sync + Send + 'static {}
