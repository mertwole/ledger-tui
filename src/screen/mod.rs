use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    Frame,
};

use crate::app::StateRegistry;

pub mod asset;
pub mod device_selection;
pub mod portfolio;

pub trait Screen {
    // TODO: Make it into unified constructor.
    fn construct(&mut self, state: StateRegistry);

    fn render(&self, frame: &mut Frame<'_>);
    fn tick(&mut self) -> Option<OutgoingMessage>;

    fn deconstruct(self: Box<Self>) -> StateRegistry;
}

pub enum OutgoingMessage {
    SwitchWindow(WindowName),
    Exit,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowName {
    DeviceSelection,
    Portfolio,
    Asset,
}

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
