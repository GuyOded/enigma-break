use enigma::Enigma;
use enigma::reflectors;
use enigma::reflectors::Reflector;
use enigma::rotor::Rotor;
use enigma::rotor::rotors;
use itertools;
use log::debug;
use std::iter;

const ALPHABET_SIZE: u8 = 26;

pub struct EnigmaBreaker {
    five_choose_three_combinations: [[usize; 3]; 10],
    three_permutations: [[usize; 3]; 6],
    available_rotors: [Rotor; 5],
    available_reflectors: [Reflector; 3],
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
    pub fn new(
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

    pub fn to_enigma(&self, reflector: Reflector) -> Enigma {
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

    pub fn known_plain_text_cipher_break(&self, cipher: &str, plain: &str) -> String {
        let mut breaking_enigma_candidates: Vec<Enigma> = Vec::new();
        for reflector in self.available_reflectors {
            for combination in self.five_choose_three_combinations {
                let mut enigma = Enigma::new(
                    self.available_rotors[0].clone(),
                    self.available_rotors[1].clone(),
                    self.available_rotors[2].clone(),
                    reflector.clone(),
                );
                self.find_rotors_configuration_candidates(
                    &mut enigma,
                    &combination,
                    &cipher,
                    plain,
                )
                .iter()
                .map(|config| config.to_enigma(reflector))
                .for_each(|enigma| breaking_enigma_candidates.push(enigma));
            }
        }
        for enigma in breaking_enigma_candidates {
            let derotorized_cipher = enigma.encrypt_str(&cipher[0..plain.len()]).unwrap();
        }

        "".to_string()
    }

    fn decrypt_derotorized_using_plain(derotorized_cipher: &str, plain: &str) -> String {
        let mut letters_map = ['\0'; ALPHABET_SIZE as usize];
        for (cipher_letter, plain_letter) in iter::zip(derotorized_cipher.chars(), plain.chars()) {
            
        }

        "".to_string()
    }

    fn find_rotors_configuration_candidates(
        &self,
        enigma: &mut Enigma,
        combination: &[usize; 3],
        cipher: &str,
        plain: &str,
    ) -> Vec<EnigmaRotorConfiguration> {
        let mut possible_configurations: Vec<EnigmaRotorConfiguration> = Vec::new();
        for permutation in self.three_permutations {
            enigma.set_left_rotor(self.available_rotors[combination[permutation[0]]].clone());
            enigma.set_middle_rotor(self.available_rotors[combination[permutation[1]]].clone());
            enigma.set_right_rotor(self.available_rotors[combination[permutation[2]]].clone());

            for (left_pos, mid_pos, right_pos) in
                itertools::iproduct!(0..ALPHABET_SIZE, 0..ALPHABET_SIZE, 0..ALPHABET_SIZE)
            {
                enigma.set_left_rotor_position_from_int(left_pos);
                enigma.set_middle_rotor_position_from_int(mid_pos);
                enigma.set_right_rotor_position_from_int(right_pos);

                if enigma
                    .encrypt_str_iter(cipher)
                    .map(|r| r.unwrap())
                    .zip(plain.chars())
                    .all(|(c, p)| c != p.to_ascii_uppercase())
                {
                    possible_configurations.push(EnigmaRotorConfiguration::new(
                        combination[permutation[0]],
                        combination[permutation[1]],
                        combination[permutation[2]],
                        left_pos,
                        mid_pos,
                        right_pos,
                    ));
                    debug!("Possible configurations: {possible_configurations:#?}")
                }
            }
        }

        possible_configurations
    }
}
