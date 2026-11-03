use std::sync::Arc;

use winit::event_loop::OwnedDisplayHandle;
use winit::window::Window;

/// GPU and window surface
pub struct Gpu {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl Gpu {
    pub async fn new(window: Arc<Window>, display: OwnedDisplayHandle) -> Gpu {
        // 1. Instance
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_with_display_handle(Box::new(display))
        );

        // 2. Surface
        let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

        // 3. Adapter
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }).await.expect("No matching GPU");

        // 4. Device and Queue
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default()).await.expect("Failed to create device");
        log::info!("Selected backend: {:?}", device.adapter_info());

        // 5. Configure surface
        let size = window.inner_size();
        let config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("Selected GPU does not support this Surface");
        surface.configure(&device, &config);

        Gpu { surface, device, queue, config }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, draw: impl FnOnce(&mut wgpu::RenderPass<'_>)) {
        // get a texture
        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(f) => f,
            wgpu::CurrentSurfaceTexture::Suboptimal(f) => {
                self.surface.configure(&self.device, &self.config);
                f
            },
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => return,
            other => panic!("Surface error: {other:?}"),
        };
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // commands
        let mut encoder = self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame")
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: "clear pass".into(),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0}),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            draw(&mut pass);
        }

        self.queue.submit([encoder.finish()]);
        self.queue.present(frame);
    }

    pub fn device(&self) -> &wgpu::Device { &self.device }
    pub fn format(&self) -> wgpu::TextureFormat { self.config.format }
}