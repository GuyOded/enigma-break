use std::{
    cmp::Reverse,
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use enigma::Enigma;
use itertools::Itertools;
use log::{debug, trace};

use crate::solver::{
    CipherMetadata, enigma_settings::EnigmaRotorConfiguration, enigma_solution_utils,
};

pub(super) fn try_building_transpositions(
    enigma: &mut Enigma,
    enigma_rotor_configuration: &EnigmaRotorConfiguration,
    cipher_metadata: Arc<CipherMetadata>,
    stop_flag: Arc<AtomicBool>,
) -> Option<HashMap<char, char>> {
    let most_frequent_plain_char = cipher_metadata.letter_positions[0].0;
    let letter_positions_in_plain = &cipher_metadata.letter_positions[0].1;

    for transposition_candidate in super::FIRST_LETTER..=super::LAST_LETTER {
        trace!("Trying {most_frequent_plain_char} <---> {transposition_candidate}");

        enigma.set_left_rotor_position_from_int(enigma_rotor_configuration.left_rotor_position);
        enigma.set_middle_rotor_position_from_int(enigma_rotor_configuration.middle_rotor_position);
        enigma.set_right_rotor_position_from_int(enigma_rotor_configuration.right_rotor_position);
        enigma.set_transposition(most_frequent_plain_char, transposition_candidate);

        if let Some(transpositions) =
            enigma_solution_utils::build_potential_transposition_for_target_letter(
                enigma,
                most_frequent_plain_char,
                letter_positions_in_plain,
                Arc::clone(&stop_flag),
            )
        {
            debug!("Found transposition possibility: {transpositions:#?}");
            return Some(transpositions.clone());
        }

        enigma.clear_transpositions();
    }

    None
}

pub(super) fn build_potential_transposition_for_target_letter<'b>(
    enigma: &'b mut Enigma,
    target_letter: char,
    letter_indexes_with_corresponding_cipher_char: &Vec<(usize, char)>,
    stop_flag: Arc<AtomicBool>,
) -> Option<&'b HashMap<char, char>> {
    let mut last_letter_position = 0;

    for &(position, cipher_char) in letter_indexes_with_corresponding_cipher_char.iter() {
        if stop_flag.load(Ordering::Relaxed) {
            return None;
        }

        enigma.increment_by(position - last_letter_position);
        let untransposed_result = enigma.encrypt_char(target_letter).unwrap();

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

    stop_flag.store(true, Ordering::Relaxed);
    Some(enigma.get_transpositions())
}

pub(super) fn build_cipher_metadata(plain: &str, cipher: &str) -> CipherMetadata {
    let mut letter_index_map: HashMap<char, Vec<(usize, char)>> = HashMap::new();
    plain
        .to_ascii_uppercase()
        .chars()
        .zip(cipher.chars())
        .enumerate()
        .for_each(|(i, (plain, cipher))| {
            if let Some(indices) = letter_index_map.get_mut(&plain) {
                indices.push((i, cipher));
                return;
            }

            letter_index_map.insert(plain, vec![(i, cipher)]);
        });

    CipherMetadata {
        letter_positions: letter_index_map
            .into_iter()
            .sorted_by_key(|(_, indices)| Reverse(indices.len()))
            .collect(),
    }
}
