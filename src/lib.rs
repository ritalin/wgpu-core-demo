use winit::event_loop::EventLoop;
pub mod app;

mod runtime;

pub async fn run() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = app::App::new(&event_loop, true);

    event_loop.run_app(&mut app)?;
    Ok(())
}
