use std::cmp::Reverse;
use std::collections::HashMap;

use enigma::Enigma;
use enigma::reflectors;
use enigma::reflectors::Reflector;
use enigma::rotor::Rotor;
use enigma::rotor::rotors;
use itertools;
use itertools::Itertools;
use log::debug;
use log::trace;

use enigma_settings::EnigmaRotorConfiguration;

use crate::solver::enigma_settings::EnigmaSettings;

mod enigma_settings;
#[cfg(test)]
mod tests;

const ALPHABET_SIZE: usize = 26;
const FIRST_LETTER: char = 'A';
const LAST_LETTER: char = 'Z';

static FIVE_CHOOSE_THREE_COMBINATIONS: [[usize; 3]; 10] = [
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
];
static THREE_PERMUTATIONS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

pub struct EnigmaSolver<'a> {
    available_rotors: [Rotor; 5],
    available_reflectors: [Reflector; 3],
    cipher: &'a str,
    cipher_metadata: CipherMetadata,
}

#[derive(Debug)]
struct CipherMetadata {
    letter_positions: Vec<(char, Vec<usize>)>,
}

impl<'a> EnigmaSolver<'a> {
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
            available_reflectors: [reflector_a, reflector_b, reflector_c],
            available_rotors: [rotor_1, rotor_2, rotor_3, rotor_4, rotor_5],
            cipher,
            cipher_metadata: EnigmaSolver::build_cipher_metadata(plain),
        }
    }

    pub fn known_plain_text_cipher_break(&self) -> Option<EnigmaSettings> {
        for reflector in self.available_reflectors.iter() {
            for combination in FIVE_CHOOSE_THREE_COMBINATIONS.iter() {
                if let Some((rotor_config, transpositions)) =
                    self.find_enigma_configuration(&combination, &reflector)
                {
                    println!(
                        "{:#?}, transpositions: {:#?}, reflector: {}",
                        rotor_config, transpositions, reflector.name
                    );
                    return Some(EnigmaSettings {
                        rotor_config,
                        transpositions,
                        reflector: *reflector,
                    });
                }
            }
        }

        None
    }

    fn find_enigma_configuration(
        &self,
        combination: &[usize; 3],
        reflector: &Reflector,
    ) -> Option<(EnigmaRotorConfiguration, HashMap<char, char>)> {
        let mut enigma: Enigma;

        for permutation in THREE_PERMUTATIONS.iter() {
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
        let most_frequent_plain_char = self.cipher_metadata.letter_positions[0].0;
        let letter_positions_in_plain = &self.cipher_metadata.letter_positions[0].1;

        for transposition_candidate in FIRST_LETTER..=LAST_LETTER {
            trace!("Trying {most_frequent_plain_char} <---> {transposition_candidate}");

            enigma.set_left_rotor_position_from_int(enigma_rotor_configuration.left_rotor_position);
            enigma.set_middle_rotor_position_from_int(
                enigma_rotor_configuration.middle_rotor_position,
            );
            enigma
                .set_right_rotor_position_from_int(enigma_rotor_configuration.right_rotor_position);
            enigma.set_transposition(most_frequent_plain_char, transposition_candidate);

            if let Some(transpositions) = self.build_potential_transposition_for_target_letter(
                enigma,
                most_frequent_plain_char,
                letter_positions_in_plain,
            ) {
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
        letter_positions: &Vec<usize>,
    ) -> Option<&'b HashMap<char, char>> {
        let mut last_letter_position = 0;

        for &position in letter_positions.iter() {
            enigma.increment_by(position - last_letter_position);
            let untransposed_result = enigma.encrypt_char(target_letter).unwrap();

            let cipher_char = self.cipher.chars().nth(position).unwrap(); // TODO: zip with plain

            if untransposed_result != cipher_char {
                if enigma
                    .get_transpositions()
                    .contains_key(&untransposed_result)
                    || enigma.get_transpositions().contains_key(&cipher_char)
                {
                    trace!(
                        "d={untransposed_result}, c={cipher_char}, i={position}, {:#?}",
                        enigma.get_transpositions()
                    );
                    return None;
                }

                enigma.set_transposition(untransposed_result, cipher_char);
            }

            last_letter_position = position + 1;
        }

        Some(enigma.get_transpositions())
    }

    fn build_cipher_metadata(plain: &str) -> CipherMetadata {
        let mut letter_indices: HashMap<char, Vec<usize>> = HashMap::new();
        plain
            .to_ascii_uppercase()
            .char_indices()
            .for_each(|(i, c)| {
                if let Some(indices) = letter_indices.get_mut(&c) {
                    indices.push(i);
                    return;
                }

                letter_indices.insert(c, vec![i]);
            });

        CipherMetadata {
            letter_positions: letter_indices
                .into_iter()
                .sorted_by_key(|(_, indices)| Reverse(indices.len()))
                .collect(),
        }
    }
}
