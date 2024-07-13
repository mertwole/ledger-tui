use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};

pub mod device_selection;
pub mod portfolio;

trait EventExt {
    fn is_key_pressed(&self, code: KeyCode) -> bool;
}

impl EventExt for Event {
    fn is_key_pressed(&self, code: KeyCode) -> bool {
        let pressed_code = code;

        matches!(
            self,
            &Event::Key(KeyEvent {
                kind: KeyEventKind::Press,
                code,
                ..
            }) if code == pressed_code
        )
    }
}
