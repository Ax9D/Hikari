use hikari::{asset::AssetManager};

use hikari::g3d::*;

pub fn save_all(asset_manager: &AssetManager, only_unsaved: bool) -> anyhow::Result<()> {
    asset_manager.save_all::<hikari::g3d::Scene>(only_unsaved)?;
    asset_manager.save_all::<Texture2D>(only_unsaved)?;
    asset_manager.save_all::<hikari::core::World>(only_unsaved)?;
    asset_manager.save_all::<Texture2D>(only_unsaved)?;
    asset_manager.save_all::<TextureCube>(only_unsaved)?;
    asset_manager.save_all::<Material>(only_unsaved)?;
    asset_manager.save_all::<EnvironmentTexture>(only_unsaved)?;

    Ok(())
}