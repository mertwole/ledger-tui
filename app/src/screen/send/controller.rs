use copypasta::{ClipboardContext, ClipboardProvider};
use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};

use super::Model;
use crate::{api::ledger::LedgerApiT, screen::OutgoingMessage};

#[derive(InputMapping)]
pub enum InputEvent {
    #[key = 'q']
    #[description = "Quit application"]
    Quit,

    #[key = 'h']
    #[description = "Open/close navigation help"]
    NavigationHelp,

    #[key = 'b']
    #[description = "Return one screen back"]
    Back,

    #[key = 'p']
    #[description = "Paste a receiver address"]
    PasteAddress,

    #[key = "KeyCode::Backspace"]
    #[description = "Erase a symbol from amount"]
    EraseSymbol,

    #[key = "KeyCode::Enter"]
    #[description = "Sign and send a transaction"]
    SignAndSend,
}

pub(super) async fn process_input<L: LedgerApiT>(
    event: &Event,
    model: &mut Model<L>,
) -> Option<OutgoingMessage> {
    let input_event = InputEvent::map_event(event.clone());

    if let Some(event) = input_event {
        match event {
            InputEvent::Quit => {
                return Some(OutgoingMessage::Exit);
            }
            InputEvent::NavigationHelp => {
                model.show_navigation_help ^= true;
                return None;
            }
            InputEvent::Back => {
                return Some(OutgoingMessage::Back);
            }
            InputEvent::PasteAddress => {
                let mut ctx = ClipboardContext::new().unwrap();
                let address = ctx.get_contents().unwrap();

                model.receiver_address = Some(address);

                return None;
            }
            InputEvent::EraseSymbol => {
                let _ = model.send_amount.pop();

                return None;
            }
            InputEvent::SignAndSend => {
                model.sign_and_send_tx().await;

                return None;
            }
        }
    }

    if let Event::Key(KeyEvent {
        code: KeyCode::Char(char),
        ..
    }) = event
    {
        match char {
            '.' => {
                model.send_amount.push(*char);
            }
            char if char.is_ascii_digit() => {
                model.send_amount.push(*char);
            }
            _ => {}
        }
    }

    None
}
