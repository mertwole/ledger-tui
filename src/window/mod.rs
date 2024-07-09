use ratatui::Frame;

pub mod connection_request;
pub mod portfolio;

pub trait Window {
    fn render(&self, frame: &mut Frame<'_>);
    fn process_events(&mut self) -> ExecutionState;
}

pub enum ExecutionState {
    Continue,
    Terminate,
    SwitchWindow(Box<dyn Window>),
}
