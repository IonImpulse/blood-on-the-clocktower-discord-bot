use std::collections::*;

#[derive(Clone)]
pub enum Alignment {
    Good,
    Evil,
}

#[derive(Clone)]
pub enum ActionTime {
    OnlyFirstNight,
    EveryNight,
    EveryNightNotFirst,
    DeathNight,
    VariableNight,
    NoNight,
}

#[derive(Clone)]
pub enum CharacterType {
    Townsfolk,
    Outsider,
    Minion,
    Demon,
    Traveler,
    Fabled,
    Other,
}

#[derive(Clone)]
pub struct Character {
    name: String,
    alignment: Alignment,
    char_type: CharacterType,
    first_order_index: i32,
    order_index: i32,
    night_action: ActionTime,
}

impl Character {
    pub fn new(
        name: String,
        char_type: CharacterType,
        first_order_index: i32,
        order_index: i32,
        night_action: ActionTime,
    ) -> Self {

        let alignment: Alignment;

        match char_type {
            CharacterType::Demon => alignment = Alignment::Evil,
            CharacterType::Minion => alignment = Alignment::Evil,
            _ => alignment = Alignment::Good
        }

        Character {
            name: name,
            alignment: alignment,
            char_type: char_type,
            first_order_index: first_order_index,
            order_index: order_index,
            night_action: night_action,
        }
    }
}

#[derive(Clone)]
pub struct GameType {
    name: String,
    roles: HashMap<String, Character>,
}

impl GameType {
    pub fn new(
        name: String,
        roles: HashMap<String, Character>,
    ) -> Self {
        GameType {
            name: name,
            roles: roles,
        }
    }
}