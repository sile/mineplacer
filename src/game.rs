use crate::asset::Assets;
use pagurus::{
    event::Event, failure::OrFail, fixed_window::FixedWindow, video::VideoFrame, Result, System,
};
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
pagurus::export_wasm_functions!(Game);

const FPS: u64 = 30;
const RENDER_TIMEOUT_DURATION: Duration = Duration::from_micros(1_000_000 / FPS);

#[derive(Debug, Default)]
pub struct Game {
    assets: Assets,
    video_frame: VideoFrame,
    fixed_window: FixedWindow,
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        self.assets.load().or_fail()?;
        todo!()
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        todo!()
    }
}
