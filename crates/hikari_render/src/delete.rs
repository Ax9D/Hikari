use std::{sync::atomic::AtomicUsize};

use ash::vk;
use gpu_allocator::vulkan::Allocation;

use crate::bindless::BindlessHandle;

pub enum DeleteRequest {
    Image(vk::Image, Allocation),
    ImageView(vk::ImageView),
    Buffer(vk::Buffer, Allocation),
    BindlessImage(BindlessHandle<vk::ImageView>),
    BindlessBuffer(BindlessHandle<vk::Buffer>),
    Framebuffer(vk::Framebuffer),
    Renderpass(vk::RenderPass),
    Swapchain(vk::SwapchainKHR)
}
impl DeleteRequest {
    pub fn process(self, device: &crate::Device) -> anyhow::Result<()> {
        match self {
            DeleteRequest::Image(image, allocation) => {
                crate::image::delete_image(device, image, allocation)?;
            }
            DeleteRequest::ImageView(view) => {
                crate::image::delete_view(device, view);
            }
            DeleteRequest::Buffer(buffer, allocation) => {
                crate::buffer::delete_buffer(device, buffer, allocation)?;
            }
            DeleteRequest::BindlessImage(view) => {
                device.bindless_resources().deallocate_image(device, view);
            },
            DeleteRequest::BindlessBuffer(buffer) => {
                device.bindless_resources().deallocate_buffer(device, buffer);
            },
            DeleteRequest::Framebuffer(framebuffer) => {
                crate::framebuffer::delete(device, framebuffer);
            },
            DeleteRequest::Renderpass(renderpass) => {
                crate::renderpass::delete(device, renderpass);
            },
            DeleteRequest::Swapchain(swapchain) => {
                let extensions = device.extensions();
                let fn_ptr = extensions.swapchain.as_ref().unwrap();
                crate::swapchain::delete(fn_ptr, swapchain);
            },

        }

        Ok(())
    }
}
struct RequestPacket {
    request: DeleteRequest,
    frame_number: usize,
}

pub const DELETION_FRAME_DELAY: usize = 2;
pub struct Deleter {
    frame_number: AtomicUsize,
    deletion_queue_send: flume::Sender<RequestPacket>,
    deletion_queue_recv: flume::Receiver<RequestPacket>,
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
        self.deletion_queue_send
            .send(packet)
            .expect("Failed to send RequestPacket")
    }
    pub fn request_delete(&self, request: DeleteRequest) {
        let frame_number = self.frame_number.load(std::sync::atomic::Ordering::Acquire);

        let packet = RequestPacket {
            request,
            frame_number,
        };

        self.send_packet(packet);
    }
    pub fn is_empty(&self) -> bool {
        self.deletion_queue_recv.is_empty()
    }
    pub(crate) fn new_frame(&self, device: &crate::Device) -> anyhow::Result<()> {
        hikari_dev::profile_function!();
        let current_frame = 1 + self
            .frame_number
            .fetch_add(1, std::sync::atomic::Ordering::Release);

        let mut enqueue_again = Vec::new();

        for packet in self.deletion_queue_recv.try_iter() {
            // If more than `DELETION_FRAME_DELAY` frames have passed, delete our resource
            if current_frame - packet.frame_number >= DELETION_FRAME_DELAY {
                packet.request.process(&device)?;
            } else {
                // Enqueue again to check next frame
                enqueue_again.push(packet);
            }
        }

        for packet in enqueue_again {
            self.send_packet(packet);
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
