use pagurus::event::{Event, Key, KeyEvent};
use pagurus::failure::OrFail;
use pagurus::Game;
use pagurus_tui::TuiSystem;
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> pagurus::Result<()> {
    pagurus::io::set_println_fn(file_println).or_fail()?;

    let mut system = TuiSystem::new().or_fail()?;
    let mut game = mineplacer::game::Game::default();
    game.initialize(&mut system).or_fail()?;
    while let Ok(event) = system.next_event() {
        if matches!(event, Event::Key(KeyEvent { key: Key::Esc, .. })) {
            break;
        }
        if !game.handle_event(&mut system, event).or_fail()? {
            break;
        }
    }
    Ok(())
}

fn file_println(msg: &str) {
    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open("mineplacer.log")
        .and_then(|mut file| writeln!(file, "{}", msg));
}
