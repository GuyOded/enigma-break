use std::fs::File;

use csv::Writer;
use enigma::Enigma;
use enigma::reflectors;
use enigma::reflectors::Reflector;
use enigma::rotor::Rotor;
use enigma::rotor::rotors;
use itertools;
use itertools::Combinations;
use log::debug;

// const ALPHABET_SIZE: u8 = 26;
const FIRST_LETTER: char = 'A';
const FIRST_LETTER_ASCII_INDEX: usize = FIRST_LETTER as usize;

pub struct IoCEnigmaSolver {
    five_choose_three_combinations: [[usize; 3]; 10],
    three_permutations: [[usize; 3]; 6],
    available_rotors: [Rotor; 5],
    available_reflectors: [Reflector; 3],
}

#[derive(Debug)]
struct RotorPositions {
    left: char,
    mid: char,
    right: char,
}

impl IoCEnigmaSolver {
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
        }
    }

    pub fn solve(&self, cipher: &str) {
        let mut max: f64 = 0.0;
        for reflector in self.available_reflectors {
            for combination in self.five_choose_three_combinations {
                for permutation in self.three_permutations {
                    let mut writer = Writer::from_path(format!(
                        "./ref{}_{}{}{}.csv",
                        reflector.name,
                        combination[permutation[0]],
                        combination[permutation[1]],
                        combination[permutation[2]]
                    ))
                    .unwrap();

                    writer
                        .write_record(["ioc", "left", "mid", "right"])
                        .expect("");
                    let enigma = Enigma::new(
                        self.available_rotors[combination[permutation[0]]].clone(),
                        self.available_rotors[combination[permutation[1]]].clone(),
                        self.available_rotors[combination[permutation[2]]].clone(),
                        reflector,
                    );
                    let (biggest_ioc, rotor_positions) = self.find_highest_ioc_rotor_configuration(
                        cipher,
                        &enigma,
                        Some(&mut writer),
                    );

                    if biggest_ioc > max {
                        max = biggest_ioc;
                        debug!(
                            "Perm: {permutation:#?}, Comb: {combination:#?}, ioc: {biggest_ioc}, Positions: {rotor_positions:#?}"
                        )
                    }
                    writer.flush().expect("");
                }

                debug!("max: {max}");
            }
        }
    }

    fn find_highest_ioc_rotor_configuration(
        &self,
        cipher: &str,
        enigma: &Enigma,
        mut csv_writer: Option<&mut Writer<File>>,
    ) -> (f64, Option<RotorPositions>) {
        let mut max: f64 = 0.0;
        let mut rotor_positions: Option<RotorPositions> = None;

        for (left_pos, mid_pos, right_pos) in itertools::iproduct!('A'..='Z', 'A'..='Z', 'A'..='Z')
        {
            enigma.set_left_rotor_position_from_char(left_pos);
            enigma.set_middle_rotor_position_from_char(mid_pos);
            enigma.set_right_rotor_position_from_char(right_pos);
            let deciphered = enigma.encrypt_str(cipher).unwrap();
            let ioc = IoCEnigmaSolver::calculate_ioc(&deciphered);
            if ioc > max {
                max = ioc;
                rotor_positions = Some(RotorPositions {
                    left: left_pos,
                    mid: mid_pos,
                    right: right_pos,
                })
            }
            if let Some(writer) = csv_writer.as_mut() {
                writer
                    .write_record([
                        ioc.to_string(),
                        left_pos.to_string(),
                        mid_pos.to_string(),
                        right_pos.to_string(),
                    ])
                    .expect("");
            }
        }

        (max, rotor_positions)
    }

    pub fn calculate_ioc(text: &str) -> f64 {
        let mut letters_occurrences: [u64; 26] = [0; 26];

        text.chars().for_each(|c| {
            letters_occurrences[(c as usize - FIRST_LETTER_ASCII_INDEX) as usize] += 1
        });

        let text_len = text.len();
        let probability_normalizer: f64 = (text_len * (text_len - 1)) as f64;
        let sum = letters_occurrences
            .iter()
            .fold(0, |acc, occurrences| acc + occurrences * (occurrences - 1));

        sum as f64 / probability_normalizer
    }
}
