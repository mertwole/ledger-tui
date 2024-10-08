use ratatui::crossterm::event::{Event, KeyCode};

pub trait InputMappingT: Sized {
    fn get_mapping() -> InputMapping;

    fn map_event(event: Event) -> Option<Self>;
}

#[derive(Debug)]
pub struct InputMapping {
    pub mapping: Vec<MappingEntry>,
}

impl InputMapping {
    pub fn merge(mut self, mut other: InputMapping) -> Self {
        self.mapping.append(&mut other.mapping);
        self
    }
}

#[derive(Debug)]
pub struct MappingEntry {
    pub key: KeyCode,
    pub description: String,
}

pub trait KeyCodeConversions {
    fn convert(self) -> KeyCode;
}

impl KeyCodeConversions for char {
    fn convert(self) -> KeyCode {
        KeyCode::Char(self)
    }
}

impl KeyCodeConversions for KeyCode {
    fn convert(self) -> KeyCode {
        self
    }
}
