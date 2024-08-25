use ratatui::crossterm::event::{Event, KeyCode};

pub trait InputMappingT: Sized {
    fn get_mapping() -> InputMapping;

    fn map_event(event: Event) -> Option<Self>;
}

pub struct InputMapping {
    pub mapping: Vec<MappingEntry>,
}

impl InputMapping {
    pub fn merge(mut self, mut other: InputMapping) -> Self {
        self.mapping.append(&mut other.mapping);
        self
    }
}

pub struct MappingEntry {
    pub key: KeyCode,
    pub description: String,
}
