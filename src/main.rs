use wgpu_core_demo::app;
use winit::event_loop::EventLoop;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    pollster::block_on(run())
}

async fn run() -> Result<(), anyhow::Error> {
    let mut app = app::App::new();
    let event_loop = EventLoop::builder().build()?;

    event_loop.run_app(&mut app)?;
    Ok(())
}
