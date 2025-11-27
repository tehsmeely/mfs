use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

// pub struct InputEnabled(pub bool);

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MainAttack,
    AttackSlot1,
    AttackSlot2,
    AttackSlot3,
    AttackSlot4,
}

impl Action {
    pub fn movement_direction(self) -> Option<Dir2> {
        match self {
            Action::MoveUp => Some(Dir2::Y),
            Action::MoveDown => Some(Dir2::NEG_Y),
            Action::MoveLeft => Some(Dir2::NEG_X),
            Action::MoveRight => Some(Dir2::X),
            _ => None,
        }
    }

    pub fn all_movements() -> [Action; 4] {
        [
            Action::MoveUp,
            Action::MoveDown,
            Action::MoveLeft,
            Action::MoveRight,
        ]
    }
}

pub fn input_map() -> InputMap<Action> {
    let mut input_map = InputMap::default();
    input_map.insert(Action::MoveLeft, KeyCode::KeyA);
    input_map.insert(Action::MoveRight, KeyCode::KeyD);
    input_map.insert(Action::MoveUp, KeyCode::KeyW);
    input_map.insert(Action::MoveDown, KeyCode::KeyS);
    input_map.insert(Action::MainAttack, MouseButton::Left);
    input_map.insert(Action::AttackSlot1, KeyCode::Digit1);
    input_map.insert(Action::AttackSlot2, KeyCode::Digit2);
    input_map.insert(Action::AttackSlot3, KeyCode::Digit3);
    input_map.insert(Action::AttackSlot4, KeyCode::Digit4);
    input_map
}
