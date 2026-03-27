use std::collections::HashMap;

use enigma::reflectors::Reflector;

use crate::solver::ALPHABET_SIZE;

#[derive(Debug, PartialEq)]
pub struct EnigmaRotorConfiguration {
    pub left_rotor_index: usize,
    pub middle_rotor_index: usize,
    pub right_rotor_index: usize,
    pub left_rotor_position: usize,
    pub middle_rotor_position: usize,
    pub right_rotor_position: usize,
}

pub struct EnigmaSettings {
    pub rotor_config: EnigmaRotorConfiguration,
    pub transpositions: HashMap<char, char>,
    pub reflector: Reflector,
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
        let _ = match (left_rotor_index, middle_rotor_index, right_rotor_index) {
            (left, _, _) if left > 4 => panic!("Left rotor not in range, left={left}"),
            (_, middle, _) if middle > 4 => {
                panic!("Middle rotor not in range, middle={middle}")
            }
            (_, _, right) if right > 4 => {
                panic!("Right rotor not in range, right={right}")
            }
            _ => (),
        };
        let _ = match (
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
            left_rotor_index,
            middle_rotor_index,
            right_rotor_index,
            left_rotor_position,
            middle_rotor_position,
            right_rotor_position,
        }
    }
}
