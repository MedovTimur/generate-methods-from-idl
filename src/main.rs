extern crate generate_func;
use generate_func::generate_methods_from_idl;

struct Program;

impl Program {
    fn send_bytes(&self, from: u64, request: Vec<u8>) {
        println!("Sending bytes from {}: {:?}", from, request);
    }
}

generate_methods_from_idl!("test.idl");

fn main() {
    let program = Program;
    program.init(1, "ExampleToken".to_string(), "EXT".to_string(), 18);
    program.mint(1, 100, 10);
    program.burn(1, 100, 10);
}
