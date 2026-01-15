use enigma::Enigma;
use enigma::reflectors;
use enigma::rotor::rotors;
mod utils;

fn main() {
    let rotor_1 = rotors::create_rotor_1();
    let rotor_2 = rotors::create_rotor_2();
    let rotor_3 = rotors::create_rotor_3();
    let rotor_4 = rotors::create_rotor_4();
    let rotor_5 = rotors::create_rotor_5();
    let reflector_a = reflectors::create_reflector_a();
    let reflector_b = reflectors::create_reflector_b();
    let reflector_c = reflectors::create_reflector_c();

    // let enigma = Enigma::new(left, mid, right, reflector);

    todo!("Break the cipher!")
}
