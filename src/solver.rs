use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread::available_parallelism;

use enigma::Enigma;
use enigma::reflectors;
use enigma::reflectors::Reflector;
use enigma::rotor::Rotor;
use enigma::rotors;
use itertools;
use itertools::Itertools;
use log::debug;
use log::trace;

use enigma_settings::EnigmaRotorConfiguration;
use threadpool::ThreadPool;

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

pub struct EnigmaSolver {
    available_rotors: [Rotor; 5],
    available_reflectors: [Reflector; 3],
    cipher_metadata: CipherMetadata,
    pool: ThreadPool,
    stop_flag: Arc<AtomicBool>,
    solution: Arc<
        Mutex<
            Option<(
                EnigmaRotorConfiguration,
                HashMap<char, char>,
                Arc<Reflector>,
            )>,
        >,
    >,
}

#[derive(Debug, Clone)]
struct CipherMetadata {
    letter_positions: Vec<(char, Vec<(usize, char)>)>,
}

impl EnigmaSolver {
    pub fn new(cipher: &str, plain: &str) -> Self {
        let reflector_a = reflectors::create_reflector_a();
        let reflector_b = reflectors::create_reflector_b();
        let reflector_c = reflectors::create_reflector_c();

        let rotor_1 = rotors::create_rotor_1();
        let rotor_2 = rotors::create_rotor_2();
        let rotor_3 = rotors::create_rotor_3();
        let rotor_4 = rotors::create_rotor_4();
        let rotor_5 = rotors::create_rotor_5();

        let available_cores = available_parallelism().unwrap().get();
        let threadpool = ThreadPool::new(available_cores - 1);

        Self {
            available_reflectors: [reflector_a, reflector_b, reflector_c],
            available_rotors: [rotor_1, rotor_2, rotor_3, rotor_4, rotor_5],
            cipher_metadata: EnigmaSolver::build_cipher_metadata(plain, cipher),
            pool: threadpool,
            stop_flag: Arc::new(AtomicBool::from(false)),
            solution: Arc::new(Mutex::new(None)),
        }
    }

    pub fn known_plain_text_cipher_break(&self) -> Option<EnigmaSettings> {
        for reflector in self.available_reflectors.iter() {
            for combination in FIVE_CHOOSE_THREE_COMBINATIONS.iter() {
                self.find_enigma_configuration(&combination, &reflector);
            }
        }

        self.pool.join();

        let data = self.solution.lock().unwrap();
        if let Some((rotor_config, transpositions, reflector)) = (*data).clone() {
            debug!(
                "{:#?}, transpositions: {:#?}, reflector: {}",
                rotor_config, transpositions, reflector.name
            );
            return Some(EnigmaSettings {
                rotor_config,
                transpositions,
                reflector: (*reflector).clone(),
            });
        }

        None
    }

    fn find_enigma_configuration(&self, combination: &[usize; 3], reflector: &Reflector) {
        let cipher_metadata = Arc::new(self.cipher_metadata.clone());
        let reflector_name = reflector.name;
        if self.stop_flag.load(Ordering::Relaxed) {
            return;
        }

        for permutation in THREE_PERMUTATIONS.iter() {
            let left_rotor_index = combination[permutation[0]];
            let middle_rotor_index = combination[permutation[1]];
            let right_rotor_index = combination[permutation[2]];

            let mut enigma = Enigma::new(
                self.available_rotors[combination[permutation[0]]].clone(),
                self.available_rotors[combination[permutation[1]]].clone(),
                self.available_rotors[combination[permutation[2]]].clone(),
                *reflector,
            );

            let solution_config = Arc::clone(&self.solution);
            let cipher_metadata_clone = Arc::clone(&cipher_metadata);
            let stop_flag = Arc::clone(&self.stop_flag);
            let reflector_arc = Arc::new(reflector.clone());

            self.pool.execute(move || {
                for (i, (left_pos, mid_pos, right_pos)) in
                    itertools::iproduct!(0..ALPHABET_SIZE, 0..ALPHABET_SIZE, 0..ALPHABET_SIZE)
                        .enumerate()
                {
                    if stop_flag.load(Ordering::Relaxed) {
                        return;
                    }
                    let currently_tested_config = EnigmaRotorConfiguration::new(
                        left_rotor_index,
                        middle_rotor_index,
                        right_rotor_index,
                        left_pos,
                        mid_pos,
                        right_pos,
                    );

                    let transpositions = EnigmaSolver::try_building_transpositions(
                        &mut enigma,
                        &currently_tested_config,
                        Arc::clone(&cipher_metadata_clone),
                        Arc::clone(&stop_flag),
                    );

                    if let Some(transpositions) = transpositions {
                        let mut data = solution_config.lock().unwrap();
                        *data = Some((
                            currently_tested_config,
                            transpositions,
                            Arc::clone(&reflector_arc),
                        ));
                        return;
                    }

                    if i % 2000 == 0 {
                        debug!(
                            "Testing current config: {currently_tested_config:#?}, reflector: {}",
                            reflector_name
                        );
                    }
                }
            });
        }
    }

    fn try_building_transpositions(
        enigma: &mut Enigma,
        enigma_rotor_configuration: &EnigmaRotorConfiguration,
        cipher_metadata: Arc<CipherMetadata>,
        stop_flag: Arc<AtomicBool>,
    ) -> Option<HashMap<char, char>> {
        let most_frequent_plain_char = cipher_metadata.letter_positions[0].0;
        let letter_positions_in_plain = &cipher_metadata.letter_positions[0].1;

        for transposition_candidate in FIRST_LETTER..=LAST_LETTER {
            trace!("Trying {most_frequent_plain_char} <---> {transposition_candidate}");

            enigma.set_left_rotor_position_from_int(enigma_rotor_configuration.left_rotor_position);
            enigma.set_middle_rotor_position_from_int(
                enigma_rotor_configuration.middle_rotor_position,
            );
            enigma
                .set_right_rotor_position_from_int(enigma_rotor_configuration.right_rotor_position);
            enigma.set_transposition(most_frequent_plain_char, transposition_candidate);

            if let Some(transpositions) =
                EnigmaSolver::build_potential_transposition_for_target_letter(
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

    fn build_potential_transposition_for_target_letter<'b>(
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

    fn build_cipher_metadata(plain: &str, cipher: &str) -> CipherMetadata {
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
}
