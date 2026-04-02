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
use log::debug;
mod consts;
mod enigma_solver_utils;

use enigma_settings::{EnigmaRotorConfiguration, EnigmaSettings};
use threadpool::ThreadPool;

mod enigma_settings;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
struct CipherMetadata {
    letter_positions: Vec<(char, Vec<(usize, char)>)>,
}

const ALPHABET_SIZE: usize = 26;
const FIRST_LETTER: char = 'A';
const LAST_LETTER: char = 'Z';

pub struct MultiThreadedEnigmaSolver {
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

pub struct EnigmaSolver {
    available_rotors: [Rotor; 5],
    available_reflectors: [Reflector; 3],
    cipher_metadata: CipherMetadata,
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

        Self {
            available_reflectors: [reflector_a, reflector_b, reflector_c],
            available_rotors: [rotor_1, rotor_2, rotor_3, rotor_4, rotor_5],
            cipher_metadata: enigma_solver_utils::build_cipher_metadata(plain, cipher),
        }
    }

    pub fn solve(&self) -> Option<EnigmaSettings> {
        for reflector in self.available_reflectors.iter() {
            for combination in consts::FIVE_CHOOSE_THREE_COMBINATIONS.iter() {
                if let Some((rotor_config, transpositions)) =
                    self.find_enigma_configuration(&combination, &reflector)
                {
                    debug!(
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

        for permutation in consts::THREE_PERMUTATIONS.iter() {
            enigma = Enigma::new(
                self.available_rotors[combination[permutation[0]]].clone(),
                self.available_rotors[combination[permutation[1]]].clone(),
                self.available_rotors[combination[permutation[2]]].clone(),
                *reflector,
            );
            for (i, (left_pos, mid_pos, right_pos)) in
                itertools::iproduct!(0..ALPHABET_SIZE, 0..ALPHABET_SIZE, 0..ALPHABET_SIZE)
                    .enumerate()
            {
                let currently_tested_config = EnigmaRotorConfiguration::new(
                    combination[permutation[0]],
                    combination[permutation[1]],
                    combination[permutation[2]],
                    left_pos,
                    mid_pos,
                    right_pos,
                );

                let transpositions = enigma_solver_utils::build_transpositions(
                    &mut enigma,
                    &currently_tested_config,
                    enigma_solver_utils::MetadataEnum::Metadata(&self.cipher_metadata),
                    None,
                );

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
}

impl MultiThreadedEnigmaSolver {
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
            cipher_metadata: enigma_solver_utils::build_cipher_metadata(plain, cipher),
            pool: threadpool,
            stop_flag: Arc::new(AtomicBool::from(false)),
            solution: Arc::new(Mutex::new(None)),
        }
    }

    pub fn solve(&self) -> Option<EnigmaSettings> {
        for reflector in self.available_reflectors.iter() {
            for combination in consts::FIVE_CHOOSE_THREE_COMBINATIONS.iter() {
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

        for permutation in consts::THREE_PERMUTATIONS.iter() {
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

                    let transpositions = enigma_solver_utils::build_transpositions(
                        &mut enigma,
                        &currently_tested_config,
                        enigma_solver_utils::MetadataEnum::ArcMetadata(&Arc::clone(
                            &cipher_metadata_clone,
                        )),
                        Some(Arc::clone(&stop_flag)),
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
}
