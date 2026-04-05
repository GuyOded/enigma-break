use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;

use enigma::{
    Enigma, reflectors::Reflector, reflectors::ReflectorType as OuterReflectorType, rotor::Rotor,
};
use enigma::{reflectors, rotors};
use serde::{Deserialize, Serialize};

use crate::solver::ALPHABET_SIZE;

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct EnigmaRotorConfiguration {
    pub left_rotor_index: usize,
    pub middle_rotor_index: usize,
    pub right_rotor_index: usize,
    pub left_rotor_position: usize,
    pub middle_rotor_position: usize,
    pub right_rotor_position: usize,
}

#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub enum ReflectorType {
    ReflectorA,
    ReflectorB,
    ReflectorC,
}

impl From<ReflectorType> for Reflector {
    fn from(value: ReflectorType) -> Self {
        match value {
            ReflectorType::ReflectorA => reflectors::create_reflector_a(),
            ReflectorType::ReflectorB => reflectors::create_reflector_b(),
            ReflectorType::ReflectorC => reflectors::create_reflector_c(),
        }
    }
}

impl From<Reflector> for ReflectorType {
    fn from(value: Reflector) -> Self {
        match value.typ {
            OuterReflectorType::ReflectorA => ReflectorType::ReflectorA,
            OuterReflectorType::ReflectorB => ReflectorType::ReflectorB,
            OuterReflectorType::ReflectorC => ReflectorType::ReflectorC,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnigmaSettings {
    pub rotor_config: EnigmaRotorConfiguration,
    pub transpositions: HashMap<char, char>,
    pub reflector_type: ReflectorType,
}

impl FromStr for EnigmaSettings {
    type Err = Box<dyn Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim_start().starts_with("{") {
            Ok(serde_json::from_str(s)?)
        } else {
            Ok(ron::from_str(s)?)
        }
    }
}

impl From<EnigmaSettings> for Enigma {
    fn from(value: EnigmaSettings) -> Self {
        let mut enigma = Enigma::new(
            rotor_from_index(value.rotor_config.left_rotor_index - 1),
            rotor_from_index(value.rotor_config.middle_rotor_index - 1),
            rotor_from_index(value.rotor_config.right_rotor_index - 1),
            value.reflector_type.into(),
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
