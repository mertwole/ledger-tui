use ratatui::crossterm::event::{Event, KeyCode};

pub trait InputMappingT: Sized {
    fn get_mapping(&self) -> InputMapping<Self>;

    fn map_event(&self, event: Event) -> Option<Self>;
}

pub struct InputMapping<M: InputMappingT> {
    pub mapping: Vec<MappingEntry<M>>,
}

pub struct MappingEntry<M: InputMappingT> {
    pub key: KeyCode,
    pub description: String,
    pub event: M,
}
