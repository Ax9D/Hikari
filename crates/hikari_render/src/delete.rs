use std::sync::{atomic::AtomicUsize};

use ash::vk;
use gpu_allocator::vulkan::Allocation;

pub enum DeleteRequest {
    VkImage(vk::Image, Allocation),
    VkImageView(vk::ImageView),
    VkSampler(vk::Sampler),
    VkBuffer(vk::Buffer, Allocation),
}
impl DeleteRequest {
    pub fn process(self, device: &crate::Device) -> anyhow::Result<()> {
        match self {
            DeleteRequest::VkImage(image, allocation) => {
                crate::image::delete_image(device, image, allocation)?;
            },
            DeleteRequest::VkImageView(view) => {
                crate::image::delete_image_view(device, view);
            },
            DeleteRequest::VkSampler(sampler) => {
                crate::image::delete_sampler(device, sampler);
            },
            DeleteRequest::VkBuffer(buffer, allocation) => {
                crate::buffer::delete_buffer(device, buffer, allocation)?;
            },
        }

        Ok(())
    }
}
struct RequestPacket {
    request: DeleteRequest,
    frame_number: usize
}

pub const DELETION_FRAME_DELAY: usize = 2;
pub struct Deleter {
    frame_number: AtomicUsize,
    deletion_queue_send: flume::Sender<RequestPacket>,
    deletion_queue_recv: flume::Receiver<RequestPacket>
}

impl Deleter {
    pub fn new() -> Self {
        let (deletion_queue_send, deletion_queue_recv) = flume::unbounded();
        Self {
            frame_number: AtomicUsize::new(0),
            deletion_queue_send,
            deletion_queue_recv,
        }
    }
    fn send_packet(&self, packet: RequestPacket) {
        self.deletion_queue_send.send(packet).expect("Failed to send RequestPacket")
    }
    pub fn request_delete(&self, request: DeleteRequest) {
        let frame_number = self.frame_number.load(std::sync::atomic::Ordering::Relaxed);

        let packet = RequestPacket {
            request,
            frame_number
        };

        self.send_packet(packet);
    }
    pub fn is_empty(&self) -> bool {
        self.deletion_queue_recv.is_empty()
    }
    pub(crate) fn new_frame(&self, device: &crate::Device) -> anyhow::Result<()> {
        hikari_dev::profile_function!();
        let current_frame = 1 + self.frame_number.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        for packet in self.deletion_queue_recv.drain() {
            // If more than `DELETION_FRAME_DELAY` frames have passed, delete our resource
            if current_frame - packet.frame_number >= DELETION_FRAME_DELAY {
                packet.request.process(&device)?;
            } else {
                // Enqueue again to check next frame
                self.send_packet(packet);
            }
        }

        Ok(())
    }
    pub(crate) fn exit(&self, device: &crate::Device) -> anyhow::Result<()> {
        for packet in self.deletion_queue_recv.try_iter() {
            packet.request.process(&device)?;
        }

        Ok(())
    }
}

#[test]
fn delete_once() -> anyhow::Result<()> {
    use crate::*;
    let gfx = Gfx::headless(GfxConfig::default())?;
    let device = gfx.device();
    let deleter = device.deleter();
    {
    let image = SampledImage::with_dimensions(device, 20, 20, 1, 1, ImageConfig::default());
    }
    assert!(device.new_frame().is_ok());

    assert!(!deleter.is_empty());

    assert!(device.new_frame().is_ok());

    assert!(deleter.is_empty());
    Ok(())
}