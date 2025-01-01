// Run the code with `cargo run --release --bin fhe-keygen` at root directory
use confidential_token::fhe_key::fhe_key_gen;
use serde_json::json;
use std::{env, fs, path};

// For timing
use std::time::Instant;

fn main() {
    let start = Instant::now();

    // generate FHE keys
    let key_config = fhe_key_gen();

    let genesis_config = json!({
        "tokens": [],
        "fhe_public_key": key_config.public_key,
        "fhe_server_key": key_config.server_key,
    });

    // get the root path and join with the key directory
    let root_path = env::current_dir().unwrap();
    let cyferio_genesis_path = path::Path::new(&root_path).join("test-data/genesis/cyferio");
    let mock_genesis_path = path::Path::new(&root_path).join("test-data/genesis/mock");
    let key_path = path::Path::new(&root_path).join("test-data/keys");

    // Ensure directories exist
    fs::create_dir_all(&cyferio_genesis_path).unwrap();
    fs::create_dir_all(&mock_genesis_path).unwrap();

    // write the genesis file to both directories
    fs::write(
        cyferio_genesis_path.join("confidential_token.json"),
        genesis_config.to_string(),
    )
    .unwrap();

    fs::write(
        mock_genesis_path.join("confidential_token.json"),
        genesis_config.to_string(),
    )
    .unwrap();

    println!(
        "[Init] FHE Keys generated and serialized in {:?}\n[Init] Public key and server key are stored in:\n  - {:?}\n  - {:?}",
        start.elapsed(),
        cyferio_genesis_path.join("confidential_token.json"),
        mock_genesis_path.join("confidential_token.json")
    );

    // store the private key for debug usage
    let fhe_private_key = json!({
        "fhe_private_key": key_config.private_key,
    });

    std::fs::write(
        key_path.join("private_key.json"),
        fhe_private_key.to_string(),
    )
    .unwrap();
    println!(
        "[Init] Private key for debugging stored in {:?}",
        key_path.join("private_key.json")
    );
}
