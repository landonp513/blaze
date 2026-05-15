use crate::App;

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
            wgpu::CurrentSurfaceTexture::Outdated => {
                anyhow::bail!("Outdated");
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
