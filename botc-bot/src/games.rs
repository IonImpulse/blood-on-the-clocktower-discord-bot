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

impl ActionTime {
    pub fn as_str(&self) -> &str {
       match *self {
        ActionTime::OnlyFirstNight => return "Only the first night",
        ActionTime::EveryNight => return "Every night",
        ActionTime::EveryNightNotFirst => return "Every night but the first",
        ActionTime::DeathNight => return "Only their death night",
        ActionTime::VariableNight => return "Some nights",
        ActionTime::NoNight => return "Never",
       }
    }
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
    Decoy,
}

impl CharacterType {
    pub fn as_str(&self) -> &str {
       match *self {
        CharacterType::Townsfolk => return "Townsfolk",
        CharacterType::Outsider => return "Outsider",
        CharacterType::Minion => return "Minion",
        CharacterType::Demon => return "Demon",
        CharacterType::Traveler => return "Traveler",
        CharacterType::Fabled => return "Fabled",
        CharacterType::Other => return "Other",
        CharacterType::Decoy => return "Decoy"
       }
    }
}

#[derive(Clone)]
pub struct Character {
    pub name: String,
    pub alignment: Alignment,
    pub char_type: CharacterType,
    pub char_type_str: String,
    pub first_order_index: i32,
    pub order_index: i32,
    pub night_action: ActionTime,
    pub decoy_character: Option<DecoyCharacter>,
}

#[derive(Clone)]
pub struct DecoyCharacter {
    pub name: String,
    pub alignment: Alignment,
    pub char_type: CharacterType,
    pub char_type_str: String,
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

        let char_type_str = String::from(char_type.as_str());

        Character {
            name: name,
            alignment: alignment,
            char_type: char_type,
            char_type_str: char_type_str,
            first_order_index: first_order_index,
            order_index: order_index,
            night_action: night_action,
            decoy_character: None,
        }
    }
    
    pub fn add_decoy(character: Character, decoy: DecoyCharacter) -> Character {
        Character {
            name: character.name,
            alignment: character.alignment,
            char_type: character.char_type,
            char_type_str: character.char_type_str,
            first_order_index: character.first_order_index,
            order_index: character.order_index,
            night_action: character.night_action,
            decoy_character: Some(decoy),
        }
    }
    pub fn get_string(&self) -> String {
        return format!("{: <18}| {: <15}| {: <25}", self.name, self.char_type.as_str(), self.night_action.as_str())
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

    pub fn get_character(&self, character_name: String) -> Character {
        return self.roles.get(&character_name).unwrap().clone()
    }

    pub fn get_all_characters(&self) -> Vec<Character> {
        let mut char_list: Vec<Character> = Vec::new();

        for character in self.roles.clone() {
            char_list.push(character.1);
        }

        return char_list
    }

    pub fn get_name(&self) -> String {
        return self.name.clone()
    }
}