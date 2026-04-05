use std::collections::HashMap;

use enigma::rotors;
use enigma::{Enigma, reflectors::Reflector, rotor::Rotor};

use crate::solver::ALPHABET_SIZE;

#[derive(Debug, PartialEq, Clone)]
pub struct EnigmaRotorConfiguration {
    pub left_rotor_index: usize,
    pub middle_rotor_index: usize,
    pub right_rotor_index: usize,
    pub left_rotor_position: usize,
    pub middle_rotor_position: usize,
    pub right_rotor_position: usize,
}

#[derive(Debug, Clone)]
pub struct EnigmaSettings {
    pub rotor_config: EnigmaRotorConfiguration,
    pub transpositions: HashMap<char, char>,
    pub reflector: Reflector,
}

impl From<EnigmaSettings> for Enigma {
    fn from(value: EnigmaSettings) -> Self {
        let mut enigma = Enigma::new(
            rotor_from_index(value.rotor_config.left_rotor_index - 1),
            rotor_from_index(value.rotor_config.middle_rotor_index - 1),
            rotor_from_index(value.rotor_config.right_rotor_index - 1),
            value.reflector,
        );

        enigma.set_left_rotor_position_from_int(value.rotor_config.left_rotor_position);
        enigma.set_middle_rotor_position_from_int(value.rotor_config.middle_rotor_position);
        enigma.set_right_rotor_position_from_int(value.rotor_config.right_rotor_position);

        for (first, second) in value.transpositions {
            enigma.set_transposition(first, second);
        }

        enigma
    }
}

fn rotor_from_index(index: usize) -> Rotor {
    match index {
        0 => rotors::create_rotor_1(),
        1 => rotors::create_rotor_2(),
        2 => rotors::create_rotor_3(),
        3 => rotors::create_rotor_4(),
        4 => rotors::create_rotor_5(),
        _ => panic!("Index '{index}' is not between 0-4. Only 5 rotors are available."),
    }
}

impl EnigmaRotorConfiguration {
    pub fn new(
        left_rotor_index: usize,
        middle_rotor_index: usize,
        right_rotor_index: usize,
        left_rotor_position: usize,
        middle_rotor_position: usize,
        right_rotor_position: usize,
    ) -> Self {
        match (left_rotor_index, middle_rotor_index, right_rotor_index) {
            (0..4, 0..4, 0..4) => (),
            _ => panic!(
                "Rotors indices must be in the range 1-5, (left, middle, right)={:?}",
                (left_rotor_index, middle_rotor_index, right_rotor_index)
            ),
        };
        match (
            left_rotor_position,
            middle_rotor_position,
            right_rotor_position,
        ) {
            (left, _, _) if left >= ALPHABET_SIZE => {
                panic!("Left position out of bounds, left={left}")
            }
            (_, middle, _) if middle >= ALPHABET_SIZE => {
                panic!("Middle position out of bounds, middle={middle} ")
            }
            (_, _, right) if right >= ALPHABET_SIZE => {
                panic!("Right position out of bounds, right={right}")
            }
            _ => (),
        };

        Self {
            left_rotor_index: left_rotor_index + 1,
            middle_rotor_index: middle_rotor_index + 1,
            right_rotor_index: right_rotor_index + 1,
            left_rotor_position,
            middle_rotor_position,
            right_rotor_position,
        }
    }
}
