use crate::tag;
use crate::{asset::Assets, model::Model, view::Window};
use pagurus::event::TimeoutEvent;
use pagurus::image::{Canvas, Color};
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
    window: Window,
    model: Model,
}

impl Game {
    fn render<S: System>(&mut self, system: &mut S) -> Result<()> {
        let mut canvas = Canvas::new(&mut self.video_frame);
        canvas.fill_color(Color::BLACK);
        canvas
            .mask_region(self.fixed_window.canvas_region())
            .fill_color(Color::RED);
        system.video_draw(self.video_frame.as_ref());
        Ok(())
    }
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        self.assets.load().or_fail()?;
        self.fixed_window = FixedWindow::new(Window::WINDOW_SIZE);
        system.clock_set_timeout(tag::RENDERING_TIMEOUT, RENDER_TIMEOUT_DURATION);
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        let mut event = self.fixed_window.handle_event(event);
        match &event {
            Event::Window(_) => {
                self.video_frame = VideoFrame::new(system.video_init(self.fixed_window.size()));
            }
            Event::Terminating => return Ok(false),
            Event::Timeout(TimeoutEvent {
                tag: tag::RENDERING_TIMEOUT,
                ..
            }) => {
                self.render(system).or_fail()?;
                system.clock_set_timeout(tag::RENDERING_TIMEOUT, RENDER_TIMEOUT_DURATION);
            }
            _ => {}
        }
        Ok(true)
    }
}
