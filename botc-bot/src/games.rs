use std::collections::*;

pub enum Alignment {
    Good,
    Evil,
}

pub enum ActionTime {
    OnlyFirstNight,
    EveryNight,
    EveryNightNotFirst,
    DeathNight,
    VariableNight,
    NoNight,
}

pub enum CharacterType {
    Townsfolk,
    Outsider,
    Minion,
    Demon,
    Traveler,
    Fabled,
    Other,
}

pub struct Role {
    name: String,
    alignment: Alignment,
    char_type: CharacterType,
    first_order_index: i32,
    order_index: i32,
    night_action: ActionTime,
}

impl Role {
    fn new(
        name: String,
        alignment: Alignment,
        char_type: CharacterType,
        first_order_index: i32,
        order_index: i32,
        night_action: ActionTime,
    ) -> Self {

        let mut alignment = Alignment::Good;

        match char_type {
            CharacterType::Demon => alignment = Alignment::Evil,

        }
        Role {
            name: name,
            alignment: alignment,
            char_type: char_type,
            first_order_index: first_order_index,
            order_index: order_index,
            night_action: night_action,
        }
    }
}

pub struct GameType {
    name: String,
    roles: HashMap<String, Role>,
}
