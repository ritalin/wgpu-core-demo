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
    texture_coords: [f32; 2],
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
                    format:  wgpu::wgt::VertexFormat::Float32x2,
                    offset: std::mem::offset_of!(Self, texture_coords) as wgpu::wgt::BufferAddress,
                    shader_location: 1,
                },
            ]),
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], texture_coords: [0.4131759, 0.00759614], }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], texture_coords: [0.0048659444, 0.43041354], }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], texture_coords: [0.28081453, 0.949397], }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], texture_coords: [0.85967, 0.84732914], }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], texture_coords: [0.9414737, 0.2652641], }, // E
];

const INDICES: &[u32] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];
