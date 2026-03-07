use std::collections::HashMap;

use enigma::Enigma;
use enigma::reflectors;
use enigma::reflectors::Reflector;
use enigma::rotor::Rotor;
use enigma::rotor::rotors;
use itertools;
use log::debug;

const ALPHABET_SIZE: u8 = 26;
// const FIRST_LETTER: char = 'A';
// const FIRST_LETTER_ASCII_INDEX: usize = FIRST_LETTER as usize;

pub struct EnigmaBreaker {
    five_choose_three_combinations: [[usize; 3]; 1],
    three_permutations: [[usize; 3]; 1],
    available_rotors: [Rotor; 5],
    available_reflectors: [Reflector; 1],
}

#[derive(Debug)]
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

    fn to_enigma(&self, reflector: Reflector) -> Enigma {
        let mut left_rotor = Self::rotor_index_to_rotor(self.left_rotor_index);
        left_rotor.set_position_from_int(self.left_rotor_position);

        let mut middle_rotor = Self::rotor_index_to_rotor(self.middle_rotor_index);
        middle_rotor.set_position_from_int(self.middle_rotor_position);

        let mut right_rotor = Self::rotor_index_to_rotor(self.right_rotor_index);
        right_rotor.set_position_from_int(self.right_rotor_position);

        Enigma::new(left_rotor, middle_rotor, right_rotor, reflector)
    }

    fn rotor_index_to_rotor(index: usize) -> Rotor {
        match index {
            0 => rotors::create_rotor_1(),
            1 => rotors::create_rotor_2(),
            2 => rotors::create_rotor_3(),
            3 => rotors::create_rotor_4(),
            4 => rotors::create_rotor_5(),
            _ => panic!("Rotor index out of range, index={index}"),
        }
    }
}

impl EnigmaBreaker {
    pub fn new() -> Self {
        let reflector_a = reflectors::create_reflector_a();
        let reflector_b = reflectors::create_reflector_b();
        let reflector_c = reflectors::create_reflector_c();

        let rotor_1 = rotors::create_rotor_1();
        let rotor_2 = rotors::create_rotor_2();
        let rotor_3 = rotors::create_rotor_3();
        let rotor_4 = rotors::create_rotor_4();
        let rotor_5 = rotors::create_rotor_5();

        Self {
            five_choose_three_combinations: [[0, 1, 2]],
            three_permutations: [[1, 0, 2]],
            available_reflectors: [reflector_b],
            available_rotors: [rotor_1, rotor_2, rotor_3, rotor_4, rotor_5],
        }
    }

    pub fn known_plain_text_cipher_break(&self, cipher: &str, plain: &str) -> String {
        for reflector in self.available_reflectors {
            for combination in self.five_choose_three_combinations {
                self.find_rotors_configuration_candidates(&combination, &cipher, plain, &reflector);
            }
        }

        "".to_string()
    }

    fn find_rotors_configuration_candidates(
        &self,
        combination: &[usize; 3],
        cipher: &str,
        plain: &str,
        reflector: &Reflector,
    ) -> Vec<(EnigmaRotorConfiguration, [char; 26])> {
        let mut max_aligned_letters: usize = 0;
        let mut best_transpositions_candidate;
        let mut second_char;
        let mut config_candidate: Option<EnigmaRotorConfiguration>;

        for permutation in self.three_permutations {
            for (i, (left_pos, mid_pos, right_pos)) in
                itertools::iproduct!(0..ALPHABET_SIZE, 0..ALPHABET_SIZE, 0..ALPHABET_SIZE)
                    .enumerate()
            {
                let currently_tested_config = EnigmaRotorConfiguration::new(
                    combination[permutation[0]],
                    combination[permutation[1]],
                    combination[permutation[2]],
                    1,
                    5,
                    17,
                );

                if i % 1000 == 0 {
                    debug!("Testing current config: {currently_tested_config:#?}");
                }

                let (possible_transpositions, second, aligned_letters) =
                    EnigmaBreaker::build_possible_transposition(
                        cipher,
                        plain,
                        &currently_tested_config,
                        reflector,
                        'A',
                        'B',
                        'C',
                    );

                if max_aligned_letters >= aligned_letters {
                    continue;
                }

                max_aligned_letters = aligned_letters;
                best_transpositions_candidate = possible_transpositions;
                second_char = second;
                config_candidate = Some(currently_tested_config);
                debug!(
                        "Found possible configuration {config_candidate:#?}
                        \nTranspositions {best_transpositions_candidate:#?}, {second_char} with {max_aligned_letters} aligned letters
                        \nMax Aligned Letters: {max_aligned_letters}"
                    );
            }
        }

        Vec::new()
    }

    fn build_possible_transposition(
        original_cipher: &str,
        plain: &str,
        enigma_rotor_configuration: &EnigmaRotorConfiguration,
        reflector: &Reflector,
        first_letter_to_transpose: char,
        second_letter_to_transpose: char,
        third_letter_to_transpose: char,
    ) -> (char, char, usize) {
        let mut max_aligned_letters = 0;
        let mut first_char_corresponding_to_max_aligned_letters = 'A';
        let mut second_char_corresponding_to_max_aligned_letters = 'A';
        let mut third_char_corresponding_to_max_aligned_letters = 'A';
        let mut enigma = enigma_rotor_configuration.to_enigma(*reflector);
        for letter in 'A'..='Z' {
            for second_transposition_letter in 'A'..='Z' {
                for third_transposition_letter in 'A'..='Z' {
                    enigma.set_transposition(first_letter_to_transpose, letter);
                    if second_letter_to_transpose == letter
                        && first_letter_to_transpose == second_transposition_letter
                    {
                        continue;
                    }

                    enigma
                        .set_transposition(second_letter_to_transpose, second_transposition_letter);
                    enigma.set_transposition(third_letter_to_transpose, third_transposition_letter);

                    let aligned_letters = EnigmaBreaker::count_aligned_letters(
                        enigma.encrypt_str_iter(plain).map(|r| r.unwrap()),
                        original_cipher,
                    );

                    if aligned_letters > max_aligned_letters {
                        max_aligned_letters = aligned_letters;
                        first_char_corresponding_to_max_aligned_letters = letter;
                        second_char_corresponding_to_max_aligned_letters =
                            second_transposition_letter;
                        third_char_corresponding_to_max_aligned_letters =
                            third_transposition_letter;
                    }

                    EnigmaBreaker::set_enigma_state_by_transpositions_and_rotor_config(
                        &mut enigma,
                        enigma_rotor_configuration,
                        None,
                    );
                }
            }
        }

        debug!(
            "{:?}",
            (
                first_char_corresponding_to_max_aligned_letters,
                second_char_corresponding_to_max_aligned_letters,
                third_char_corresponding_to_max_aligned_letters,
                max_aligned_letters
            )
        );
        (
            first_char_corresponding_to_max_aligned_letters,
            second_char_corresponding_to_max_aligned_letters,
            max_aligned_letters,
        )
    }

    fn set_enigma_state_by_transpositions_and_rotor_config(
        enigma: &mut Enigma,
        rotor_config: &EnigmaRotorConfiguration,
        transpositions: Option<&HashMap<char, char>>,
    ) {
        enigma.set_left_rotor_position_from_int(rotor_config.left_rotor_position);
        enigma.set_right_rotor_position_from_int(rotor_config.right_rotor_position);
        enigma.set_middle_rotor_position_from_int(rotor_config.middle_rotor_position);
        enigma.clear_transpositions();

        transpositions.map(|map| {
            map.iter().for_each(|(&key, &value)| {
                enigma.set_transposition(key, value);
            });
        });
    }

    fn count_aligned_letters(plain_candidate: impl Iterator<Item = char>, plain: &str) -> usize {
        plain_candidate
            .zip(plain.chars())
            .filter(|&(deciphered, plain)| deciphered == plain.to_ascii_uppercase())
            .count()
    }
}
