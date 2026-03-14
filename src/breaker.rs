use std::cmp::Reverse;
use std::collections::HashMap;

use enigma::Enigma;
use enigma::reflectors;
use enigma::reflectors::Reflector;
use enigma::rotor::Rotor;
use enigma::rotor::rotors;
use itertools;
use log::debug;
use log::trace;

const ALPHABET_SIZE: u8 = 26;
const FIRST_LETTER: char = 'A';
const FIRST_LETTER_ASCII_INDEX: usize = FIRST_LETTER as usize;

pub struct EnigmaBreaker<'a> {
    five_choose_three_combinations: [[usize; 3]; 10],
    three_permutations: [[usize; 3]; 6],
    available_rotors: [Rotor; 5],
    available_reflectors: [Reflector; 3],
    cipher: &'a str,
    plain: &'a str,
    cipher_metadata: CipherMetadata,
}

#[derive(Debug)]
struct CipherMetadata {
    letter_frequency_order: Vec<(char, u32)>,
}

#[derive(Debug, PartialEq)]
struct EnigmaRotorConfiguration {
    left_rotor_index: usize,
    middle_rotor_index: usize,
    right_rotor_index: usize,
    left_rotor_position: u8,
    middle_rotor_position: u8,
    right_rotor_position: u8,
}

impl EnigmaRotorConfiguration {
    fn new(
        left_rotor_index: usize,
        middle_rotor_index: usize,
        right_rotor_index: usize,
        left_rotor_position: u8,
        middle_rotor_position: u8,
        right_rotor_position: u8,
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

impl<'a> EnigmaBreaker<'a> {
    pub fn new(cipher: &'a str, plain: &'a str) -> Self {
        let reflector_a = reflectors::create_reflector_a();
        let reflector_b = reflectors::create_reflector_b();
        let reflector_c = reflectors::create_reflector_c();

        let rotor_1 = rotors::create_rotor_1();
        let rotor_2 = rotors::create_rotor_2();
        let rotor_3 = rotors::create_rotor_3();
        let rotor_4 = rotors::create_rotor_4();
        let rotor_5 = rotors::create_rotor_5();

        Self {
            five_choose_three_combinations: [
                [0, 1, 2],
                [0, 1, 3],
                [0, 1, 4],
                [0, 2, 3],
                [0, 2, 4],
                [0, 3, 4],
                [1, 2, 3],
                [1, 2, 4],
                [1, 3, 4],
                [2, 3, 4],
            ],
            three_permutations: [
                [0, 1, 2],
                [0, 2, 1],
                [1, 0, 2],
                [1, 2, 0],
                [2, 0, 1],
                [2, 1, 0],
            ],
            available_reflectors: [reflector_a, reflector_b, reflector_c],
            available_rotors: [rotor_1, rotor_2, rotor_3, rotor_4, rotor_5],
            cipher,
            plain,
            cipher_metadata: EnigmaBreaker::build_cipher_metadata(plain),
        }
    }

    pub fn known_plain_text_cipher_break(&self) {
        for reflector in self.available_reflectors {
            for combination in self.five_choose_three_combinations {
                if let Some((enigma_config, transpositions)) =
                    self.find_enigma_configuration(&combination, &reflector)
                {
                    println!(
                        "{:#?}, transpositions: {:#?}, reflector: {}",
                        enigma_config, transpositions, reflector.name
                    );
                    return;
                }
            }
        }
    }

    fn find_enigma_configuration(
        &self,
        combination: &[usize; 3],
        reflector: &Reflector,
    ) -> Option<(EnigmaRotorConfiguration, HashMap<char, char>)> {
        let mut enigma: Enigma;

        for permutation in self.three_permutations {
            for (i, (left_pos, mid_pos, right_pos)) in
                itertools::iproduct!(0..ALPHABET_SIZE, 0..ALPHABET_SIZE, 0..ALPHABET_SIZE)
                    .enumerate()
            {
                enigma = Enigma::new(
                    self.available_rotors[combination[permutation[0]]].clone(),
                    self.available_rotors[combination[permutation[1]]].clone(),
                    self.available_rotors[combination[permutation[2]]].clone(),
                    *reflector,
                );

                let currently_tested_config = EnigmaRotorConfiguration::new(
                    combination[permutation[0]],
                    combination[permutation[1]],
                    combination[permutation[2]],
                    left_pos,
                    mid_pos,
                    right_pos,
                );

                let transpositions =
                    self.try_building_transpositions(&mut enigma, &currently_tested_config);

                if let Some(transpositions) = transpositions {
                    return Some((currently_tested_config, transpositions));
                }

                if i % 2000 == 0 {
                    debug!(
                        "Testing current config: {currently_tested_config:#?}, reflector: {}",
                        reflector.name
                    );
                }
            }
        }

        None
    }

    fn try_building_transpositions<'b>(
        &self,
        enigma: &mut Enigma,
        enigma_rotor_configuration: &EnigmaRotorConfiguration,
    ) -> Option<HashMap<char, char>> {
        let currently_tested_char = self.cipher_metadata.letter_frequency_order[0].0;

        for transposition_candidate in FIRST_LETTER..'Z' {
            trace!("Trying {currently_tested_char} <---> {transposition_candidate}");

            enigma.set_left_rotor_position_from_int(enigma_rotor_configuration.left_rotor_position);
            enigma.set_middle_rotor_position_from_int(
                enigma_rotor_configuration.middle_rotor_position,
            );
            enigma
                .set_right_rotor_position_from_int(enigma_rotor_configuration.right_rotor_position);
            enigma.set_transposition(currently_tested_char, transposition_candidate);

            if let Some(transpositions) =
                self.build_potential_transposition_for_target_letter(enigma, currently_tested_char)
            {
                debug!("Found transposition possibility: {transpositions:#?}");
                return Some(transpositions.clone());
            }

            enigma.clear_transpositions();
        }

        None
    }

    fn build_potential_transposition_for_target_letter<'b>(
        &self,
        enigma: &'b mut Enigma,
        target_letter: char,
    ) -> Option<&'b HashMap<char, char>> {
        for (i, c) in self.plain.to_ascii_uppercase().char_indices() {
            if c == target_letter {
                let untransposed_result = enigma.encrypt_char(c).unwrap();
                let cipher_char = self.cipher.chars().nth(i).unwrap();

                match (
                    untransposed_result != cipher_char,
                    (enigma
                        .get_transpositions()
                        .contains_key(&untransposed_result)
                        || enigma.get_transpositions().contains_key(&cipher_char)),
                ) {
                    (true, true) => {
                        trace!(
                            "d={untransposed_result}, c={cipher_char}, i={i}, {:#?}",
                            enigma.get_transpositions()
                        );
                        return None;
                    }
                    (true, false) => enigma.set_transposition(untransposed_result, cipher_char),
                    (false, _) => (),
                }
                continue;
            }
            let _ = enigma.encrypt_char(c);
        }

        Some(enigma.get_transpositions())
    }

    fn build_cipher_metadata(plain: &str) -> CipherMetadata {
        let mut letter_frequencies: [u32; ALPHABET_SIZE as usize] = [0; ALPHABET_SIZE as usize];
        let mut letter_frequency_order: Vec<(char, u32)> = Vec::new();
        plain.to_ascii_uppercase().chars().for_each(|c| {
            letter_frequencies[(c as u8) as usize - FIRST_LETTER_ASCII_INDEX] += 1;
        });

        letter_frequencies
            .iter()
            .enumerate()
            .for_each(|(i, &freq)| {
                letter_frequency_order
                    .push(((i as u8 + FIRST_LETTER_ASCII_INDEX as u8) as char, freq));
            });

        letter_frequency_order.sort_by_key(|&(_, freq)| Reverse(freq));

        CipherMetadata {
            letter_frequency_order,
        }
    }
}
