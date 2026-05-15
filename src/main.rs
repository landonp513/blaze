use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use std::sync::Arc;
use wgpu::{
    BackendOptions, Backends, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints, PresentMode,
    RequestAdapterOptions, Surface,
};

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
        if self.gpu.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("blaze")
            .with_resizable(true);

        let instance_desc = InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::from_build_config(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::from_env_or_default(),
            display: Some(Box::new(event_loop.owned_display_handle())),
        };

        let instance = Instance::new(instance_desc);
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter_options = RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        };
        let adapter = pollster::block_on(instance.request_adapter(&adapter_options))
            .expect("no adapter found");

        let device_desc = DeviceDescriptor {
            label: Some("gpu0"),
            required_features: Features::empty(),
            required_limits: Limits::downlevel_defaults(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::default(),
            trace: wgpu::Trace::Off,
        };
        println!("adapter: {:?}", adapter.get_info());
        let (device, queue) = pollster::block_on(adapter.request_device(&device_desc))
            .unwrap_or_else(|e| panic!("failed to get device: {e:?}"));

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width, size.height)
            .expect("surface not supported");
        config.present_mode = PresentMode::Fifo;
        surface.configure(&device, &config);

        self.window = Some(window);
        self.gpu = Some(GpuState {
            surface: surface,
            device: device,
            queue: queue,
            config: config,
        })
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {}
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.config.width = size.width.max(1);
                    gpu.config.height = size.height.max(1);
                    gpu.surface.configure(&gpu.device, &gpu.config);
                }
            }
            _ => (),
        }
    }
}

impl App {
    fn render(&mut self) -> anyhow::Result<()> {
        self.window?.request_redraw();
        let gpu = self.gpu.as_mut().unwrap();

        let output = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                gpu.surface.configure(&gpu.device, &gpu.config);
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("Lost Device");
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        Ok(())
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
