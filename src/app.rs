use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::gpu::Gpu;
use crate::render::Renderer;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes().with_title("Cellular Automation");
            let window = Arc::new(event_loop.create_window(attrs).expect("Failed to create window"));
            
            let gpu = pollster::block_on(Gpu::new(window.clone(), event_loop.owned_display_handle()));
            let renderer = Renderer::new(gpu.device(), gpu.format());
            window.request_redraw();
            self.window = Some(window);
            self.gpu = Some(gpu);
            self.renderer = Some(renderer);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    )
    {
        let (Some(gpu), Some(renderer)) = (self.gpu.as_mut(), self.renderer.as_ref()) else { return };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() && event.physical_key == PhysicalKey::Code(KeyCode::Escape) {
                    event_loop.exit();
                }
            },
            WindowEvent::Resized(size) => {
                gpu.resize(size.width, size.height);
            },
            WindowEvent::RedrawRequested => gpu.render(|pass| renderer.draw(pass)),
            _ => {}
        }
    }
}