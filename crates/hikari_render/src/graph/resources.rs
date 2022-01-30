use std::{
    collections::{hash_map::Values, HashMap},
    sync::Arc,
};

use crate::texture::SampledImage;

use super::{storage::Storage, Handle, ImageSize};

pub type HandlesIter<'a> = Values<'a, String, Handle<SampledImage>>;

pub struct GraphResources {
    images: Storage<SampledImage>,
    img_handles: HashMap<String, Handle<SampledImage>>,
}
impl GraphResources {
    pub fn new() -> Self {
        let image_storage = Storage::new();

        Self {
            images: image_storage,
            img_handles: HashMap::default(),
        }
    }
    pub fn add_image(
        &mut self,
        name: String,
        image: SampledImage,
        size: ImageSize,
    ) -> Handle<SampledImage> {
        if self.img_handles.get(&name).is_none() {
            let handle = self.images.add(image, size);

            self.img_handles.insert(name, handle.clone());
            handle
        } else {
            panic!("Image with name {} already exists", name);
        }
    }

    #[inline]
    pub fn get_image(&self, handle: &Handle<SampledImage>) -> Option<&SampledImage> {
        self.images.get(handle)
    }
    #[inline]
    pub fn get_image_with_size(
        &self,
        handle: &Handle<SampledImage>,
    ) -> Option<(&SampledImage, &ImageSize)> {
        self.images.get_with_metadata(handle)
    }

    pub fn get_image_handle(&self, name: &str) -> Option<Handle<SampledImage>> {
        self.img_handles.get(name).cloned()
    }
    pub fn image_handles(&self) -> HandlesIter {
        self.img_handles.values()
    }

    // #[inline]
    // pub(crate) fn get_image_list(&self) -> &ResourceList<SampledImage> {
    //     self.storage.get_list().unwrap()
    // }

    // pub(crate) fn replace_image(
    //     &mut self,
    //     handle: &Handle<SampledImage>,
    //     new_image: SampledImage,
    // ) -> Option<SampledImage> {
    //     self.images
    //     .get_mut(handle)
    //     .map(|image| std::mem::replace(image, new_image))
    // }
    pub fn resize_images(
        &mut self,
        device: &Arc<crate::Device>,
        new_width: u32,
        new_height: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for handle in self.img_handles.values() {
            let (image, size) = self.images.get_with_metadata_mut(handle).unwrap();
            let config = *image.config();
            let (new_width, new_height) = size.get_physical_size((new_width, new_height));
            let new_image = SampledImage::with_dimensions(device, new_width, new_height, config)?;

            let old_image = std::mem::replace(image, new_image);
        }

        Ok(())
    }
}
