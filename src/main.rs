use enigma::Enigma;
use enigma::reflectors;
use enigma::rotor::rotors;

fn main() {
    let left = rotors::create_rotor_1();
    let mid = rotors::create_rotor_2();
    let right = rotors::create_rotor_3();
    let reflector = reflectors::create_reflector_a();

    let enigma = Enigma::new(left, mid, right, reflector);

    todo!("Break the cipher!")
}
