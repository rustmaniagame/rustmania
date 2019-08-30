#![warn(clippy::pedantic)]

#[cfg(feature = "dx12")]
use gfx_backend_dx12 as gfx_backend;
#[cfg(feature = "metal")]
use gfx_backend_metal as gfx_backend;
#[cfg(feature = "vulkan")]
use gfx_backend_vulkan as gfx_backend;
#[cfg(feature = "default")]
use gfx_backend_vulkan as gfx_backend;

use arrayvec::ArrayVec;

use core::mem::ManuallyDrop;

use winit::{dpi::LogicalSize, CreationError, EventsLoop, Window, WindowBuilder};

use gfx_hal::{
    command::{ClearColor, ClearValue, CommandBuffer, MultiShot, Primary},
    format::{Aspects, ChannelType, Format, Swizzle},
    image::{Extent, Layout, SubresourceRange, ViewKind},
    pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc},
    pool::{CommandPool, CommandPoolCreateFlags},
    pso::PipelineStage,
    window::{Surface, Swapchain},
    Adapter, Backend, Device, Features, Gpu, Graphics, Instance, PhysicalDevice, QueueFamily,
    QueueGroup, Submission, SwapchainConfig,
};

#[derive(Debug)]
pub struct WinitState {
    pub events_loop: EventsLoop,
    pub window: Window,
}

#[derive(Debug)]
pub struct Context {
    instance: ManuallyDrop<gfx_backend::Instance>,
    surface: <gfx_backend::Backend as Backend>::Surface,
    adapters: Vec<Adapter<gfx_backend::Backend>>,
    devices: Vec<DeviceData>,
    command_pools: Vec<CommandPool<gfx_backend::Backend, Graphics>>,
    command_buffers: Vec<CommandBuffer<gfx_backend::Backend, Graphics, MultiShot, Primary>>,
}

#[derive(Debug)]
struct DeviceData {
    adapter_index: usize,
    device: gfx_backend::Device,
    queue: QueueGroup<gfx_backend::Backend, Graphics>,
    swapchains: Vec<SwapchainData>,
    render_passes: Vec<<gfx_backend::Backend as Backend>::RenderPass>,
}

#[derive(Debug)]
struct SwapchainData {
    swapchain: <gfx_backend::Backend as Backend>::Swapchain,
    backbuffer: Vec<<gfx_backend::Backend as Backend>::Image>,
    config: SwapchainConfig,
    fences: Option<Vec<<gfx_backend::Backend as Backend>::Fence>>,
    available_semaphores: Option<Vec<<gfx_backend::Backend as Backend>::Semaphore>>,
    finished_semaphores: Option<Vec<<gfx_backend::Backend as Backend>::Semaphore>>,
    current_frame: usize,
    image_views: Option<Vec<<gfx_backend::Backend as Backend>::ImageView>>,
    framebuffers: Vec<<gfx_backend::Backend as Backend>::Framebuffer>,
}

impl Default for WinitState {
    /// Makes an 800x600 window with the `WINDOW_NAME` value as the title.
    /// ## Panics
    /// If a `CreationError` occurs.
    fn default() -> Self {
        Self::new(
            "",
            LogicalSize {
                width: 800.0,
                height: 600.0,
            },
        )
        .expect("Could not create a window!")
    }
}

impl WinitState {
    /// Constructs a new `EventsLoop` and `Window` pair.
    ///
    /// The specified title and size are used, other elements are default.
    /// ## Failure
    /// It's possible for the window creation to fail. This is unlikely.
    pub fn new<T: Into<String>>(title: T, size: LogicalSize) -> Result<Self, CreationError> {
        let events_loop = EventsLoop::new();
        let output = WindowBuilder::new()
            .with_title(title)
            .with_dimensions(size)
            .build(&events_loop);
        output.map(|window| Self {
            events_loop,
            window,
        })
    }
}

impl Context {
    pub fn build(window: &Window, name: &str) -> Result<Self, &'static str> {
        let mut context = Self::from_window(window, name);
        context.add_device()?;
        context.add_swapchain(0)?;
        context.add_semaphors(0, 0)?;
        context.devices[0].add_render_pass()?;
        context.devices[0].add_image_views(0)?;
        context.devices[0].add_framebuffers(0, 0)?;

        context.command_pools.push(unsafe {
            context.devices[0]
                .device
                .create_command_pool_typed(
                    &context.devices[0].queue,
                    CommandPoolCreateFlags::RESET_INDIVIDUAL,
                )
                .map_err(|_| "Could not create the raw command pool!")?
        });

        context.command_buffers =
            context.devices[0].create_command_buffers(&mut context.command_pools[0]);

        Ok(context)
    }
    pub fn from_window(window: &Window, name: &str) -> Self {
        let raw_instance = gfx_backend::Instance::create(name, 1);
        let surface = raw_instance.create_surface(window);
        let adapters = raw_instance
            .enumerate_adapters()
            .into_iter()
            .map(|mut a| {
                a.queue_families = a
                    .queue_families
                    .into_iter()
                    .filter(|qf| qf.supports_graphics() && surface.supports_queue_family(qf))
                    .collect();
                a
            })
            .filter(|a| !a.queue_families.is_empty())
            .collect::<Vec<_>>();
        Self {
            instance: ManuallyDrop::new(raw_instance),
            surface,
            adapters,
            devices: vec![],
            command_pools: vec![],
            command_buffers: vec![],
        }
    }
    fn add_device(&mut self) -> Result<(), &'static str> {
        let (index, Gpu { device, mut queues }, family) = self
            .adapters
            .iter()
            .enumerate()
            .find_map(|(index, a)| {
                a.queue_families.iter().find_map(|qf| unsafe {
                    a.physical_device
                        .open(&[(&qf, &[1.0; 1])], Features::empty())
                        .ok()
                        .map(|gpu| (index, gpu, qf))
                })
            })
            .ok_or("Failed to find a working queue or something")?;
        let queue_group = queues
            .take::<Graphics>(family.id())
            .ok_or("Couldn't take ownership of the QueueGroup")?;
        if queue_group.queues.is_empty() {
            return Err("The QueueGroup did not have any CommandQueues available!");
        };
        self.devices
            .push(DeviceData::from(index, device, queue_group));
        Ok(())
    }
    fn add_swapchain(&mut self, device_index: usize) -> Result<(), &'static str> {
        let DeviceData {
            adapter_index,
            device,
            ..
        } = self
            .devices
            .get(device_index)
            .ok_or("Failed to get device")?;
        let (surface_capabilities, preferred_formats, present_modes) = self
            .surface
            .compatibility(&self.adapters[*adapter_index].physical_device);
        let &present_mode = {
            use gfx_hal::window::PresentMode::*;
            [Mailbox, Fifo, Relaxed, Immediate]
                .iter()
                .find(|pm| present_modes.contains(pm))
                .ok_or("No PresentMode values specified!")?
        };
        let format = match preferred_formats {
            None => Format::Rgba8Srgb,
            Some(formats) => match formats
                .iter()
                .find(|format| format.base_format().1 == ChannelType::Srgb)
                .cloned()
            {
                Some(srgb_format) => srgb_format,
                None => formats
                    .get(0)
                    .cloned()
                    .ok_or("Preferred format list was empty!")?,
            },
        };
        let swapchain_config = SwapchainConfig {
            present_mode,
            composite_alpha: surface_capabilities.composite_alpha,
            format,
            extent: *surface_capabilities.extents.end(),
            image_count: *surface_capabilities.image_count.start(),
            image_layers: 1,
            image_usage: surface_capabilities.usage,
        };
        let (swapchain, backbuffer) = unsafe {
            device
                .create_swapchain(&mut self.surface, swapchain_config.clone(), None)
                .map_err(|_| "Failed to create the swapchain!")?
        };
        self.devices[0].swapchains.push(SwapchainData::from(
            swapchain,
            backbuffer,
            swapchain_config,
            None,
            None,
            None,
            None,
            vec![],
        ));
        Ok(())
    }
    fn add_semaphors(
        &mut self,
        device_index: usize,
        swapchain_index: usize,
    ) -> Result<(), &'static str> {
        self.devices
            .get_mut(device_index)
            .ok_or("No device with this index")?
            .add_semaphors(swapchain_index)
    }
    pub fn clear(&mut self, color: [f32; 4]) -> Result<(), &'static str> {
        self.devices[0].clear_frame(color, &mut self.command_buffers)
    }
}

impl DeviceData {
    fn from(
        adapter_index: usize,
        device: gfx_backend::Device,
        queue: QueueGroup<gfx_backend::Backend, Graphics>,
    ) -> Self {
        Self {
            adapter_index,
            device,
            queue,
            swapchains: vec![],
            render_passes: vec![],
        }
    }
    //make this index safe
    fn add_semaphors(&mut self, swapchain_index: usize) -> Result<(), &'static str> {
        let image_count = self.swapchains[swapchain_index].config.image_count;
        let device = &self.device;
        self.swapchains[swapchain_index].fences = Some(
            (0..image_count)
                .map(|_| {
                    device
                        .create_fence(true)
                        .map_err(|_| "Could not create a fence!")
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        self.swapchains[swapchain_index].available_semaphores = Some(
            (0..image_count)
                .map(|_| {
                    device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        self.swapchains[swapchain_index].finished_semaphores = Some(
            (0..image_count)
                .map(|_| {
                    device
                        .create_semaphore()
                        .map_err(|_| "Could not create a semaphore!")
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(())
    }

    fn add_render_pass(&mut self) -> Result<(), &'static str> {
        self.render_passes.push({
            let color_attachment = Attachment {
                format: Some(self.swapchains[0].config.format),
                samples: 1,
                ops: AttachmentOps {
                    load: AttachmentLoadOp::Clear,
                    store: AttachmentStoreOp::Store,
                },
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::Present,
            };
            let subpass = SubpassDesc {
                colors: &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: None,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };
            unsafe {
                self.device
                    .create_render_pass(&[color_attachment], &[subpass], &[])
                    .map_err(|_| "Couldn't create a render pass!")?
            }
        });
        Ok(())
    }
    fn add_image_views(&mut self, swapchain_index: usize) -> Result<(), &'static str> {
        self.swapchains[swapchain_index].image_views = Some(
            self.swapchains[swapchain_index]
                .backbuffer
                .iter()
                .map(|image| unsafe {
                    self.device
                        .create_image_view(
                            &image,
                            ViewKind::D2,
                            self.swapchains[swapchain_index].config.format,
                            Swizzle::NO,
                            SubresourceRange {
                                aspects: Aspects::COLOR,
                                levels: 0..1,
                                layers: 0..1,
                            },
                        )
                        .map_err(|_| "Couldn't create the image_view for the image!")
                })
                .collect::<Result<Vec<_>, &str>>()?,
        );
        Ok(())
    }
    fn add_framebuffers(
        &mut self,
        swapchain_index: usize,
        render_pass_index: usize,
    ) -> Result<(), &'static str> {
        unsafe {
            self.swapchains[swapchain_index]
                .create_framebuffers(&self.device, &self.render_passes[render_pass_index])?
        };
        Ok(())
    }
    fn create_command_buffers(
        &mut self,
        command_pool: &mut CommandPool<gfx_backend::Backend, Graphics>,
    ) -> Vec<CommandBuffer<gfx_backend::Backend, Graphics, MultiShot, Primary>> {
        self.swapchains[0]
            .framebuffers
            .iter()
            .map(|_| command_pool.acquire_command_buffer::<MultiShot>())
            .collect()
    }
    fn reset_current_fence(&self, swapchain_index: usize) -> Result<(), &'static str> {
        unsafe {
            self.device
                .wait_for_fence(
                    &self.swapchains[swapchain_index]
                        .fences
                        .as_ref()
                        .ok_or("Could not get fence")?
                        [self.swapchains[swapchain_index].current_frame],
                    u64::max_value(),
                )
                .map_err(|_| "Failed to wait on the fence!")?;
            self.device
                .reset_fence(
                    &self.swapchains[swapchain_index]
                        .fences
                        .as_ref()
                        .ok_or("Could not get fence")?
                        [self.swapchains[swapchain_index].current_frame],
                )
                .map_err(|_| "Couldn't reset the fence!")?;
        }
        Ok(())
    }
    fn clear_frame(
        &mut self,
        color: [f32; 4],
        command_buffers: &mut [CommandBuffer<gfx_backend::Backend, Graphics, MultiShot, Primary>],
    ) -> Result<(), &'static str> {
        // Advance the frame _before_ we start using the `?` operator
        self.swapchains[0].advance_frame();

        let (i_u32, i_usize) = unsafe { self.swapchains[0].get_current_image()? };

        self.reset_current_fence(0)?;

        // RECORD COMMANDS
        unsafe {
            let buffer = &mut command_buffers[i_usize];
            let clear_values = [ClearValue::Color(ClearColor::Sfloat(color))];
            buffer.begin(false);
            buffer.begin_render_pass_inline(
                &self.render_passes[0],
                &self.swapchains[0].framebuffers[i_usize],
                self.swapchains[0].config.extent.to_extent().rect(),
                clear_values.iter(),
            );
            buffer.finish();
        }

        // SUBMISSION AND PRESENT
        let command_buffers = &command_buffers[i_usize..=i_usize];
        let wait_semaphores: ArrayVec<[_; 1]> = [(
            &self.swapchains[0]
                .available_semaphores
                .as_ref()
                .ok_or("fail")?[self.swapchains[0].current_frame],
            PipelineStage::COLOR_ATTACHMENT_OUTPUT,
        )]
        .into();
        let signal_semaphores: ArrayVec<[_; 1]> = [&self.swapchains[0]
            .finished_semaphores
            .as_ref()
            .ok_or("fail")?[self.swapchains[0].current_frame]]
        .into();
        // yes, you have to write it twice like this. yes, it's silly.
        let present_wait_semaphores: ArrayVec<[_; 1]> = [&self.swapchains[0]
            .finished_semaphores
            .as_ref()
            .ok_or("fail")?[self.swapchains[0].current_frame]]
        .into();
        let submission = Submission {
            command_buffers,
            wait_semaphores,
            signal_semaphores,
        };
        let the_command_queue = &mut self.queue.queues[0];
        unsafe {
            the_command_queue.submit(
                submission,
                Some(
                    &self.swapchains[0].fences.as_ref().ok_or("")?
                        [self.swapchains[0].current_frame],
                ),
            );
            self.swapchains[0]
                .swapchain
                .present(the_command_queue, i_u32, present_wait_semaphores)
                .map_err(|_| "Failed to present into the swapchain!")?
        };
        Ok(())
    }
}

impl SwapchainData {
    #![allow(clippy::too_many_arguments)]
    fn from(
        swapchain: <gfx_backend::Backend as Backend>::Swapchain,
        backbuffer: Vec<<gfx_backend::Backend as Backend>::Image>,
        config: SwapchainConfig,
        fences: Option<Vec<<gfx_backend::Backend as Backend>::Fence>>,
        available_semaphores: Option<Vec<<gfx_backend::Backend as Backend>::Semaphore>>,
        finished_semaphores: Option<Vec<<gfx_backend::Backend as Backend>::Semaphore>>,
        image_views: Option<Vec<<gfx_backend::Backend as Backend>::ImageView>>,
        framebuffers: Vec<<gfx_backend::Backend as Backend>::Framebuffer>,
    ) -> Self {
        Self {
            swapchain,
            backbuffer,
            config,
            fences,
            available_semaphores,
            finished_semaphores,
            current_frame: 0,
            image_views,
            framebuffers,
        }
    }
    unsafe fn get_current_image(&mut self) -> Result<(u32, usize), &'static str> {
        let (image_index, _check_if_you_need_this) = self
            .swapchain
            .acquire_image(
                u64::max_value(),
                Some(&self.available_semaphores.as_ref().ok_or("fail")?[self.current_frame]),
                Some(&self.fences.as_ref().ok_or("")?[self.current_frame]),
            )
            .map_err(|_| "Couldn't acquire an image from the swapchain!")?;
        Ok((image_index, image_index as usize))
    }
    unsafe fn create_framebuffers(
        &mut self,
        device: &gfx_backend::Device,
        render_pass: &<gfx_backend::Backend as Backend>::RenderPass,
    ) -> Result<(), &'static str> {
        self.framebuffers = self
            .image_views
            .as_ref()
            .ok_or("No image views on this swapchain")?
            .iter()
            .map(|image_view| {
                device
                    .create_framebuffer(
                        render_pass,
                        vec![image_view],
                        Extent {
                            width: self.config.extent.width as u32,
                            height: self.config.extent.height as u32,
                            depth: 1,
                        },
                    )
                    .map_err(|_| "Failed to create a framebuffer!")
            })
            .collect::<Result<Vec<_>, &str>>()?;
        Ok(())
    }
    fn advance_frame(&mut self) {
        self.current_frame = match &self.available_semaphores {
            Some(semaphores) => (self.current_frame + 1) % semaphores.len(),
            None => 0,
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
