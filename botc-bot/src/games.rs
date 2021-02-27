use std::collections::*;

pub enum Alignment {
    Good,
    Evil
}

pub enum ActionTime {
    FirstNight,
    EveryNight,
    DeathNight,
    VariableNight,
    NoNight,
}

pub struct Role {
    name: String,
    alignment: Alignment,
    order_index: u32,
    night_action: ActionTime,
}

impl Role {
    fn new(name: String, alignment: Alignment, order_index: u32, night_action: ActionTime) -> Self {
        Role {
            name: name,
            alignment: alignment,
            order_index: order_index,
            night_action: night_action
        }
    }
}

pub struct GameType {
    name: String,
    roles: HashMap<String, Role>
}
