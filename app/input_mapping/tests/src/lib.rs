#![cfg(test)]

use std::collections::HashMap;

use input_mapping_common::InputMappingT;
use input_mapping_derive::InputMapping;
use ratatui::crossterm::event::KeyCode;

#[derive(InputMapping)]
enum TestEnum {
    #[key = 'a']
    One,

    #[description = "test"]
    Two,

    #[allow(dead_code)]
    Nested(Nested),
}

#[derive(InputMapping)]
enum Nested {
    #[description = "four_test"]
    Four,

    Six,
}

#[test]
fn test_input_mapping_generated_as_expected() {
    let mapping = TestEnum::get_mapping();
    let mapping: HashMap<_, _> = mapping
        .mapping
        .into_iter()
        .map(|map| (map.key, map.description))
        .collect();

    assert_eq!(mapping.len(), 4);
    assert_eq!(
        mapping.get(&KeyCode::Char('f')),
        Some(&"four_test".to_string())
    );
    assert_eq!(mapping.get(&KeyCode::Char('t')), Some(&"test".to_string()));
    assert_eq!(mapping.get(&KeyCode::Char('s')), Some(&"".to_string()));
}
