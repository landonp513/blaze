mod input;
mod render;

use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use std::sync::Arc;
use wgpu::{
    BackendOptions, Backends, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, InstanceFlags, Limits, MemoryBudgetThresholds, MemoryHints, PresentMode,
    RequestAdapterOptions, util::DeviceExt,
};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    pub gpu: Option<GpuState>,
}

pub struct GpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub render_pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
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

        let pipeline = App::init_shaders(&device, &config);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(render::VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = render::VERTICES.len() as u32;

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(render::INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = render::INDICES.len() as u32;

        self.window = Some(window);
        self.gpu = Some(GpuState {
            surface: surface,
            device: device,
            queue: queue,
            config: config,
            render_pipeline: pipeline,
            vertex_buffer: vertex_buffer,
            num_vertices: num_vertices,
            index_buffer: index_buffer,
            num_indices: num_indices,
        })
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.update();
                match self.render() {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("render error: {e:?}");
                        event_loop.exit();
                    }
                };
            }
            WindowEvent::Resized(size) => {
                if let Some(gpu) = &mut self.gpu {
                    gpu.config.width = size.width.max(1);
                    gpu.config.height = size.height.max(1);
                    gpu.surface.configure(&gpu.device, &gpu.config);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
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
