use crate::tag;
use crate::{model::Model, view::Window};
use pagurus::event::{TimeoutEvent, WindowEvent};
use pagurus::image::{Canvas, Color};
use pagurus::{
    event::Event, failure::OrFail, fixed_window::FixedWindow, video::VideoFrame, Result, System,
};
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
        self.fixed_window = FixedWindow::new(Window::WINDOW_SIZE);
        self.window.load_assets().or_fail()?;
        self.model.initialize(system).or_fail()?;

        self.model.generate_board(system).or_fail()?;

        system.clock_set_timeout(tag::RENDERING_TIMEOUT, RENDER_TIMEOUT_DURATION);
        Ok(())
    }

    fn handle_event(&mut self, system: &mut S, event: Event) -> Result<bool> {
        let event = self.fixed_window.handle_event(event);
        match event {
            Event::Window(WindowEvent::RedrawNeeded { .. }) => {
                self.video_frame = VideoFrame::new(system.video_init(self.fixed_window.size()));
                self.render(system).or_fail()?;
            }
            Event::Terminating => return Ok(false),
            Event::Timeout(TimeoutEvent {
                tag: tag::RENDERING_TIMEOUT,
                ..
            }) => {
                self.render(system).or_fail()?;
                system.clock_set_timeout(tag::RENDERING_TIMEOUT, RENDER_TIMEOUT_DURATION);
            }
            _ => {
                self.window.handle_event(event, &mut self.model).or_fail()?;
            }
        }
        if self.window.take_help_button_clicked() {
            self.action_queue.push_back(Action::OpenHelp);
        }

        Ok(true)
    }

    fn query(&mut self, _system: &mut S, name: &str) -> Result<Vec<u8>> {
        match name {
            "nextAction" => {
                if let Some(action) = self.action_queue.pop_front() {
                    return Ok(serde_json::to_vec(&action).or_fail()?);
                }
            }
            _ => pagurus::todo!(),
        }
        Ok(vec![])
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Action {
    OpenHelp,
}
