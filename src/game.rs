use crate::model::Level;
use crate::tag;
use crate::{model::Model, view::Window};
use orfail::{Failure, OrFail};
use pagurus::image::{Canvas, Color};
use pagurus::{event::Event, fixed_window::FixedWindow, video::VideoFrame, Result, System};
use std::collections::VecDeque;
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
pagurus::export_wasm_functions!(Game);

const FPS: u64 = 30;
const RENDER_TIMEOUT_DURATION: Duration = Duration::from_micros(1_000_000 / FPS);

#[derive(Debug, Default)]
pub struct Game {
    video_frame: VideoFrame,
    fixed_window: FixedWindow,
    window: Window,
    model: Model,
    action_queue: VecDeque<Action>,
}

impl Game {
    fn render<S: System>(&mut self, system: &mut S) -> Result<()> {
        self.model.update_elapsed_time(system);

        let mut canvas = Canvas::new(&mut self.video_frame);
        canvas.fill_color(Color::BLACK);
        self.window
            .render(
                &mut canvas.subregion(self.fixed_window.canvas_region()),
                &self.model,
            )
            .or_fail()?;
        system.video_draw(self.video_frame.as_ref());
        Ok(())
    }
}

impl<S: System> pagurus::Game<S> for Game {
    fn initialize(&mut self, system: &mut S) -> Result<()> {
        self.model.initialize(system).or_fail()?;
        self.window.load_assets(&self.model).or_fail()?;
        self.fixed_window = FixedWindow::new(self.window.window_size(&self.model));

        system.clock_set_timeout(tag::RENDERING_TIMEOUT, RENDER_TIMEOUT_DURATION);
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        let event = self.fixed_window.handle_event(event);
        match event {
            Event::WindowResized(_) => {
                self.video_frame = VideoFrame::new(system.video_init(self.fixed_window.size()));
                self.render(system).or_fail()?;
            }
            Event::Timeout(tag::RENDERING_TIMEOUT) => {
                self.render(system).or_fail()?;
                system.clock_set_timeout(tag::RENDERING_TIMEOUT, RENDER_TIMEOUT_DURATION);
            }
            Event::Timeout(tag::START_8X15_TIMEOUT) => {
                self.model.start_game(system, Level::Small).or_fail()?;
                self.render(system).or_fail()?;
            }
            Event::Timeout(tag::START_16X30_TIMEOUT) => {
                self.model.start_game(system, Level::Large).or_fail()?;
                self.render(system).or_fail()?;
            }
            Event::Timeout(tag::START_16X30_WITH_WORMHOLE_TIMEOUT) => {
                self.model
                    .start_game(system, Level::LargeWithWormhole)
                    .or_fail()?;
                self.render(system).or_fail()?;
            }
            _ => {
                self.window.handle_event(event, &mut self.model).or_fail()?;
            }
        }
        if self.window.take_help_button_clicked() {
            self.action_queue.push_back(Action::OpenHelp);
        }
        if self.window.take_start_8x15_button_clicked() {
            system.clock_set_timeout(tag::START_8X15_TIMEOUT, Duration::from_secs(0));
        }
        if self.window.take_start_16x30_button_clicked() {
            system.clock_set_timeout(tag::START_16X30_TIMEOUT, Duration::from_secs(0));
        }
        if self.window.take_start_16x30_with_wormhole_button_clicked() {
            system.clock_set_timeout(
                tag::START_16X30_WITH_WORMHOLE_TIMEOUT,
                Duration::from_secs(0),
            );
        }
        if self.window.take_start_custom_button_clicked() {
            // Reuse `START_16X30_WITH_WORMHOLE_TIMEOUT` tag for custom level
            system.clock_set_timeout(
                tag::START_16X30_WITH_WORMHOLE_TIMEOUT,
                Duration::from_secs(0),
            );
        }

        Ok(true)
    }

    fn query(&mut self, _system: &mut S, name: &str) -> Result<Vec<u8>> {
        match name {
            "nextAction" => {
                if let Some(action) = self.action_queue.pop_front() {
                    return serde_json::to_vec(&action).or_fail();
                }
            }
            _ => return Err(Failure::new(format!("unknown query: {name:?}"))),
        }
        Ok(vec![])
    }

    fn command(&mut self, system: &mut S, name: &str, data: &[u8]) -> Result<()> {
        match name {
            "setQueryString" => {
                let qs = std::str::from_utf8(data).or_fail()?;
                if let Some(level) = Level::from_qs(qs).or_fail()? {
                    self.model.set_custom_level(level);
                    self.fixed_window = FixedWindow::new(self.window.window_size(&self.model));
                    self.video_frame = VideoFrame::new(system.video_init(self.fixed_window.size()));
                }
                Ok(())
            }
            _ => Err(Failure::new(format!("unknown command: {name:?}"))),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Action {
    OpenHelp,
}
