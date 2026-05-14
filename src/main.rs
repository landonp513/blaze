use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowId},
};

use wgpu::{
    BackendOptions, Backends, CreateSurfaceError, Instance, InstanceDescriptor, InstanceFlags,
    MemoryBudgetThresholds,
};
use std::sync::Arc;

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuState>,
}

struct GpuState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("blaze")
            .with_resizable(true);

        let mut instance_desc = InstanceDescriptor {
            backends: Backends::all(),
            flags: InstanceFlags::from_build_config(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::from_env_or_default(),
            display: Some(Box::new(event_loop.owned_display_handle())),
        };

        let instance = Instance::new(instance_desc);
        let window = Arc::new(
            event_loop.create_window(attrs).unwrap()
        );
        let surface = instance.create_surface(window.clone()).unwrap();
        
        
        self.window = Some(window);
        self.gpu = Some(GpuState { surface, device: (), queue: (), config: () })
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }

            _ => (),
        }
    }
}

fn main() -> Result<(), ()> {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();

    match event_loop.run_app(&mut app) {
        Ok(()) => (),
        Err(e) => {
            eprintln!("event_loop error: {}", e);
        }
    }

    Ok(())
}
