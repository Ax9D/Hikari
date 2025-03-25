use std::collections::HashMap;
use ash::vk;

type Map<K, V> = HashMap<K, V, hikari_utils::hash::BuildHasher>;

#[derive(Clone, Copy, Default, PartialEq)]
pub struct SamplerCreateInfo {
    pub mag_filter: vk::Filter,
    pub min_filter: vk::Filter,
    pub mipmap_mode: vk::SamplerMipmapMode,
    pub address_mode_u: vk::SamplerAddressMode,
    pub address_mode_v: vk::SamplerAddressMode,
    pub address_mode_w: vk::SamplerAddressMode,
    pub mip_lod_bias: f32,
    pub anisotropy_enable: bool,
    pub max_anisotropy: f32,
    pub compare_enable: bool,
    pub compare_op: vk::CompareOp,
    pub min_lod: f32,
    pub max_lod: f32,
    pub border_color: vk::BorderColor,
    pub unnormalized_coordinates: bool,
    pub sampler_reduction_mode: Option<vk::SamplerReductionMode>
}
impl std::hash::Hash for SamplerCreateInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.mag_filter.hash(state);
        self.min_filter.hash(state);
        self.mipmap_mode.hash(state);
        self.address_mode_u.hash(state);
        self.address_mode_v.hash(state);
        self.address_mode_w.hash(state);
        self.mip_lod_bias.to_bits().hash(state);
        self.anisotropy_enable.hash(state);
        self.max_anisotropy.to_bits().hash(state);
        self.compare_enable.hash(state);
        self.compare_op.hash(state);
        self.min_lod.to_bits().hash(state);
        self.max_lod.to_bits().hash(state);
        self.border_color.hash(state);
        self.unnormalized_coordinates.hash(state);
        self.sampler_reduction_mode.hash(state);
    }
}

impl Eq for SamplerCreateInfo {}

impl SamplerCreateInfo {
    fn create_sampler(&self, device: &ash::Device) -> vk::Sampler {

        let mut reduce_info = vk::SamplerReductionModeCreateInfo::builder();

        let create_info = vk::SamplerCreateInfo::builder()
                                                    .min_filter(self.min_filter)
                                                    .mag_filter(self.mag_filter)
                                                    .mipmap_mode(self.mipmap_mode)
                                                    .address_mode_u(self.address_mode_u)
                                                    .address_mode_v(self.address_mode_v)
                                                    .address_mode_w(self.address_mode_w)
                                                    .mip_lod_bias(self.mip_lod_bias)
                                                    .anisotropy_enable(self.anisotropy_enable)
                                                    .max_anisotropy(self.max_anisotropy)
                                                    .compare_enable(self.compare_enable)
                                                    .compare_op(self.compare_op)
                                                    .min_lod(self.min_lod)
                                                    .max_lod(self.max_lod)
                                                    .border_color(self.border_color)
                                                    .unnormalized_coordinates(self.unnormalized_coordinates);
    
        let create_info = if let Some(reduction_mode) = self.sampler_reduction_mode {
            reduce_info.reduction_mode = reduction_mode;
            create_info.push_next(&mut reduce_info)
        } else {
            create_info
        };

        unsafe { device.create_sampler(&create_info, None).unwrap() }
    }
}
pub(crate) struct SamplerCache {
    device: ash::Device,
    samplers: Map<SamplerCreateInfo, vk::Sampler>
}

impl SamplerCache {
    pub fn new(device: &ash::Device) -> Self {
        Self {
            device: device.clone(),
            samplers: Default::default()
        }
    }
    pub fn get_sampler(&mut self, create_info: &SamplerCreateInfo) -> vk::Sampler {
        let create_info = *create_info;
        let sampler = self.samplers.entry(create_info).or_insert_with(|| {
            let sampler = create_info.create_sampler(&self.device);

            sampler
        });

        *sampler
    }
}

impl Drop for SamplerCache {
    fn drop(&mut self) {
        for &sampler in self.samplers.values() {
            unsafe {
                self.device.destroy_sampler(sampler, None);
            }
        }
    }
}