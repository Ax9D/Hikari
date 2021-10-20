use std::{error::Error, sync::Arc};

use graphy::Device;
// pub trait Asset {
//     fn loadFromFile<P: AsRef<str>>(renderContext: HandleMut<Context>, path: P) -> Result<Handle<Self>, Box<dyn Error>>;
// }
pub struct AssetManager {
    device: Arc<Device>,
    //textures: Vec<Arc<Texture2D>>,
}
impl AssetManager {
    pub fn new(device: &Arc<Device>) -> Self {
        Self {
            //textures,
            device: device.clone(),
        }
    }
    //     pub fn loadTexture<P: AsRef<str>>(
    //         &mut self,
    //         path: P,
    //         config: TextureConfig,
    //     ) -> Result<Arc<Texture2D>, Box<dyn Error>> {
    //         let tex = Texture2D::fromFile(path, config, &self.device)?;
    //         self.textures.push(tex.clone());
    //         Ok(tex)
    //     }
    //     // pub fn load<T: Asset, P: AsRef<str>>(&mut self, asset: T, path: P) -> Result<Handle<T>, Box<dyn Error>>{
    //     //     Asset::loadFromFile(self.renderContext, path)
    //     // }
}
