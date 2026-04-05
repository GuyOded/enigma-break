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

use crate::solver::{CipherMetadata, enigma_settings::EnigmaRotorConfiguration};

pub(super) enum MetadataEnum<'a> {
    ArcMetadata(&'a Arc<CipherMetadata>),
    Metadata(&'a CipherMetadata),
}

pub(super) fn build_transpositions(
    enigma: &mut Enigma,
    enigma_rotor_configuration: &EnigmaRotorConfiguration,
    cipher_metadata: MetadataEnum,
    stop_flag: Option<&Arc<AtomicBool>>,
) -> Option<HashMap<char, char>> {
    let (most_frequent_plain_char, letter_positions_in_plain) = match cipher_metadata {
        MetadataEnum::ArcMetadata(cipher_metadata) => (
            cipher_metadata.letter_positions[0].0,
            &cipher_metadata.letter_positions[0].1,
        ),
        MetadataEnum::Metadata(cipher_metadata) => (
            cipher_metadata.letter_positions[0].0,
            &cipher_metadata.letter_positions[0].1,
        ),
    };

    for transposition_candidate in super::FIRST_LETTER..=super::LAST_LETTER {
        trace!("Trying {most_frequent_plain_char} <---> {transposition_candidate}");

        enigma.set_left_rotor_position_from_int(enigma_rotor_configuration.left_rotor_position);
        enigma.set_middle_rotor_position_from_int(enigma_rotor_configuration.middle_rotor_position);
        enigma.set_right_rotor_position_from_int(enigma_rotor_configuration.right_rotor_position);
        enigma.set_transposition(most_frequent_plain_char, transposition_candidate);
        let transpositions_result_for_target_letter = match stop_flag {
            Some(stop_flag) => build_transpositions_by_target_letter_multithreaded(
                enigma,
                most_frequent_plain_char,
                letter_positions_in_plain,
                stop_flag,
            ),
            None => build_transpositions_by_target_letter(
                enigma,
                most_frequent_plain_char,
                letter_positions_in_plain,
            ),
        };

        if let Some(transpositions) = transpositions_result_for_target_letter {
            debug!("Found transposition possibility: {transpositions:#?}");
            return Some(transpositions.clone());
        }

        enigma.clear_transpositions();
    }

    None
}

pub(super) fn build_transpositions_by_target_letter_multithreaded<'b>(
    enigma: &'b mut Enigma,
    target_letter: char,
    letter_indexes_with_corresponding_cipher_char: &[(usize, char)],
    stop_flag: &Arc<AtomicBool>,
) -> Option<&'b HashMap<char, char>> {
    let mut last_letter_position = 0;

    for &(position, cipher_char) in letter_indexes_with_corresponding_cipher_char.iter() {
        match increment_enigma_and_transpose_new_letter(
            position,
            last_letter_position,
            enigma,
            cipher_char,
            target_letter,
        ) {
            Ok(_) => (),
            Err(TranspositionError::LetterExists) => return None,
        }

        last_letter_position = position + 1;
    }

    stop_flag.store(true, Ordering::Relaxed);
    Some(enigma.get_transpositions())
}

pub(super) fn build_transpositions_by_target_letter<'b>(
    enigma: &'b mut Enigma,
    target_letter: char,
    letter_indexes_with_corresponding_cipher_char: &[(usize, char)],
) -> Option<&'b HashMap<char, char>> {
    let mut last_letter_position = 0;

    for &(position, cipher_char) in letter_indexes_with_corresponding_cipher_char.iter() {
        match increment_enigma_and_transpose_new_letter(
            position,
            last_letter_position,
            enigma,
            cipher_char,
            target_letter,
        ) {
            Ok(_) => (),
            Err(TranspositionError::LetterExists) => return None,
        }

        last_letter_position = position + 1;
    }

    Some(enigma.get_transpositions())
}

enum TranspositionError {
    LetterExists,
}

fn increment_enigma_and_transpose_new_letter(
    position: usize,
    last_letter_position: usize,
    enigma: &mut Enigma,
    corresponding_cipher_char: char,
    target_letter: char,
) -> Result<(), TranspositionError> {
    enigma.increment_by(position - last_letter_position);
    let untransposed_result = enigma.encrypt_char(target_letter).unwrap();

    if untransposed_result != corresponding_cipher_char {
        let transpositions = enigma.get_transpositions();
        if transpositions.contains_key(&untransposed_result)
            || transpositions.contains_key(&corresponding_cipher_char)
        {
            trace!(
                "d={untransposed_result}, c={corresponding_cipher_char}, i={position}, {:#?}",
                enigma.get_transpositions()
            );
            return Err(TranspositionError::LetterExists);
        }

        enigma.set_transposition(untransposed_result, corresponding_cipher_char);
    }

    Ok(())
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
