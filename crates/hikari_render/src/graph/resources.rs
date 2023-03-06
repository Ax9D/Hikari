use std::{
    collections::{hash_map::Iter, HashMap},
    sync::Arc,
};

use crate::{image::SampledImage, Buffer};

use super::{
    storage::{ErasedHandle, GenericBufferStorage, Storage},
    GpuHandle, ImageSize,
};

pub type HandlesIter<'a> = Iter<'a, String, GpuHandle<SampledImage>>;
pub type BufferHandlesIter<'a> = Iter<'a, String, ErasedHandle>;
pub struct GraphResources {
    images: Storage<SampledImage>,
    buffers: GenericBufferStorage,
    img_handles: HashMap<String, GpuHandle<SampledImage>>,
    buffer_handles: HashMap<String, ErasedHandle>,
}
impl GraphResources {
    pub fn new() -> Self {
        let image_storage = Storage::new();
        let buffers = GenericBufferStorage::new();

        Self {
            images: image_storage,
            buffers,
            img_handles: HashMap::default(),
            buffer_handles: HashMap::default(),
        }
    }
    pub fn add_image(
        &mut self,
        name: String,
        image: SampledImage,
        size: ImageSize,
    ) -> GpuHandle<SampledImage> {
        if self.img_handles.get(&name).is_none() {
            let handle = self.images.add(image, size);

            self.img_handles.insert(name, handle.clone());
            handle
        } else {
            panic!("Image with name {} already exists", name);
        }
    }
    pub fn add_buffer<B: Buffer + Send + Sync + 'static>(
        &mut self,
        name: String,
        buffer: B,
    ) -> GpuHandle<B> {
        if self.buffer_handles.get(&name).is_none() {
            let handle = self.buffers.add(buffer);

            self.buffer_handles.insert(name, handle.clone().into());

            handle
        } else {
            panic!("Buffer with name {} already exists", name);
        }
    }

    #[inline]
    pub fn get_image(&self, handle: &GpuHandle<SampledImage>) -> Option<&SampledImage> {
        self.images.get(handle)
    }
    pub fn get_image_by_name(&self, name: &str) -> Option<&SampledImage> {
        self.get_image(&self.get_image_handle(name)?)
    }
    #[inline]
    pub fn get_image_with_size(
        &self,
        handle: &GpuHandle<SampledImage>,
    ) -> Option<(&SampledImage, &ImageSize)> {
        self.images.get_with_metadata(handle)
    }

    pub fn get_image_handle(&self, name: &str) -> Option<GpuHandle<SampledImage>> {
        self.img_handles.get(name).cloned()
    }
    pub fn image_handles(&self) -> HandlesIter {
        self.img_handles.iter()
    }

    #[inline]
    pub fn get_buffer<B: Buffer + Send + Sync + 'static>(
        &self,
        handle: &GpuHandle<B>,
    ) -> Option<&B> {
        self.buffers.get(handle)
    }
    pub fn get_buffer_by_name<B: Buffer + Send + Sync + 'static>(&self, name: &str) -> Option<&B> {
        self.buffers.get(&self.get_buffer_handle(name)?)
    }
    pub fn get_buffer_handle<B: Buffer + Send + Sync + 'static>(
        &self,
        name: &str,
    ) -> Option<GpuHandle<B>> {
        self.buffer_handles
            .get(name)
            .map(|erased| erased.clone().into_typed::<B>().unwrap())
    }
    #[inline]
    pub(crate) fn get_dyn_buffer(&self, handle: &ErasedHandle) -> Option<&dyn Buffer> {
        self.buffers.get_dyn_buffer(handle)
    }
    pub fn buffer_handles(&self) -> BufferHandlesIter {
        self.buffer_handles.iter()
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
    ) -> anyhow::Result<()> {
        for handle in self.img_handles.values() {
            let (image, size) = self.images.get_with_metadata_mut(handle).unwrap();
            let config = *image.config();
            let (new_width, new_height, new_depth) =
                size.get_physical_size_3d((new_width, new_height));
            let new_image = SampledImage::with_dimensions(
                device,
                new_width,
                new_height,
                new_depth,
                image.layers(),
                config,
            )?;

            let old_image = std::mem::replace(image, new_image);
        }

        Ok(())
    }
}
