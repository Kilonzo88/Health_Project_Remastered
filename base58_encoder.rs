use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <hex_string>", args[0]);
        return;
    }

    let hex_string = &args[1];
    let bytes = hex::decode(hex_string).expect("Invalid hex string");
    let base58_string = bs58::encode(bytes).into_string();
    println!("{}", base58_string);
}
