use winit::event_loop::EventLoop;
pub mod app;

pub async fn run() -> Result<(), anyhow::Error> {
    let mut app = app::App::new();
    let event_loop = EventLoop::builder().build()?;

    event_loop.run_app(&mut app)?;
    Ok(())
}
