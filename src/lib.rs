use std::borrow::Cow;

use winit::event_loop::EventLoop;
pub mod app;

mod runtime;
mod render;
mod wgpu_resource;

pub async fn run() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = app::App::new(&event_loop, true);

    event_loop.run_app(&mut app)?;
    Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::NoUninit, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
impl Vertex {
    fn desc() -> wgpu::wgc::pipeline::VertexBufferLayout<'static> {
        wgpu::wgc::pipeline::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::wgt::BufferAddress,
            step_mode: wgpu::wgt::VertexStepMode::Vertex,
            attributes: Cow::Borrowed(&[
                wgpu::wgt::VertexAttribute {
                    format: wgpu::wgt::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::wgt::VertexAttribute {
                    format:  wgpu::wgt::VertexFormat::Float32x3,
                    offset: std::mem::offset_of!(Self, color) as wgpu::wgt::BufferAddress,
                    shader_location: 1,
                },
            ]),
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
];

const INDICES: &[u32] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];
