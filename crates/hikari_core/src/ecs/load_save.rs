use hikari_asset::{Loader, Saver};

use crate::{Registry, World, Plugin};

pub const SUPPORTED_WORLD_EXTENSIONS: [&str; 1] = ["hworld"]; 
pub struct WorldLoader {
    registry: Registry
}

impl Loader for WorldLoader {
    fn extensions(&self) -> &[&str] {
        &SUPPORTED_WORLD_EXTENSIONS
    }

    fn load(&self, ctx: &mut hikari_asset::LoadContext) -> anyhow::Result<()> {
        let deserializer = serde_yaml::Deserializer::from_reader(ctx.reader());
        let world = World::deserialize(deserializer, &self.registry)?;

        ctx.set_asset(world);
        Ok(())
    }
}

impl Saver for WorldLoader {
    fn extensions(&self) -> &[&str] {
        &SUPPORTED_WORLD_EXTENSIONS
    }

    fn save(&self, context: &mut hikari_asset::SaveContext, writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        let asset = context.get_asset::<World>();
        let serializable_world = asset.as_serializable(&self.registry);

        serde_yaml::to_writer(writer, &serializable_world)?;
        Ok(())
    }
}

pub struct WorldLoaderPlugin;
impl Plugin for WorldLoaderPlugin {
    fn build(self, game: &mut crate::Game) {
        game.create_asset::<World>();

        let registry = game.get::<Registry>().clone();

        game.register_asset_loader::<World, WorldLoader>(WorldLoader {
            registry: registry.clone()
        });

        game.register_asset_saver::<World, WorldLoader>(WorldLoader {
            registry: registry
        });
    }
}