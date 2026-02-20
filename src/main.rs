mod breaker;
use breaker::EnigmaBreaker;
use enigma::{Enigma, reflectors, rotor::rotors};

fn main() {
    colog::init();
    let plain = "internalfleetstatusandplanningmemorandumforseniornavalcommandgeneralsituationandfleetposturewereportthatourfleetremainsdeployedinaccordancewithstandingoperationalguidanceallunitsmaintainassignedpositionswithdisciplineandconsistencytheoverallpostureemphasizesreadinesscontrolledpresenceandthepreservationofoperationalfreedomofaction";
    let cipher = "QFRDLDGYSOBGZGYYYSYCZEOBCYJHMSXIYAUUHHILYFCKTQHXSFDMSQATJWJAMZVJJHZDFNFBGLPIEJYZDUTLPLKWHVYPXJMUKWLIICWQXGCDGPYDISXKJVGCNNZCFYJVGEBGSCOKSWGNKUGHUWBEMBPKQHQVOZLLKBFEOWWUOFJPFOCRUUUTOCXXPZRJXUJUYFMHGJZXJAXTGZKGQQIWGUPGCRMXYSSBABDDWMXXFQKLUCXPSHEOPRCVBOWJPBBKURTTNKRGIWQVAPDOMAJFSAYZYGQXWTHGLCJTGCZTJMFVKDTMWCYQJYCMJMLAJXAQUVUZVKBUYLT";

    let breaker = EnigmaBreaker::new();
    breaker.known_plain_text_cipher_break(cipher, plain);

    /* let mut e = Enigma::new(
        rotors::create_rotor_2(),
        rotors::create_rotor_1(),
        rotors::create_rotor_3(),
        reflectors::create_reflector_b(),
    );

    e.set_left_rotor_position_from_int(1);
    e.set_right_rotor_position_from_int(17);
    e.set_middle_rotor_position_from_int(5);
    e.set_transposition('A', 'Z');
    e.set_transposition('C', 'M');
    e.set_transposition('D', 'R');
    e.set_transposition('E', 'H');
    e.set_transposition('F', 'P');
    e.set_transposition('G', 'L');
    e.set_transposition('I', 'S');
    e.set_transposition('J', 'N');
    e.set_transposition('K', 'W');
    e.set_transposition('O', 'X');
    e.set_transposition('U', 'V');
    e.set_transposition('Y', 'T');
    e.set_transposition('B', 'Q');

    let s = e.encrypt_str(plain);
    println!("{}", s.as_ref().unwrap()); */
}
