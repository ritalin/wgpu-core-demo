use wgpu_core_demo::run;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    pollster::block_on(run())
}
