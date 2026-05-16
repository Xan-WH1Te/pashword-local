use sha3::{Digest, Sha3_512, Shake256};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use scrypt::{scrypt, Params};

const SCRYPT_N: u8 = 15;       // log2(32768) = 15
const SCRYPT_R: u32 = 8;
const SCRYPT_P: u32 = 1;

const ALLOWED: &[u8] = b"@#$%&*._!0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const NUMBERS: &[u8] = b"1234567890";
const SYMBOLS: &[u8] = b"@#$%&*._!";

pub fn generate_pashword(
    website: &str,
    username: &str,
    secret_key: &str,
    length: u8,
) -> Result<String, String> {
    let to_hash = format!(
        r#"{{"website":"{}","username":"{}","password":"{}"}}"#,
        website, username, secret_key
    );

    // Step 1: SHA3-512 → hex string
    let sha3_hex = {
        let mut hasher = Sha3_512::new();
        Digest::update(&mut hasher, to_hash.as_bytes());
        hex::encode(hasher.finalize())
    };

    // Step 2: scrypt(N=32768, r=8, p=1, salt=website+username, dklen=32)
    // Input is hex-encoded SHA3-512 string as bytes
    let params = Params::new(SCRYPT_N, SCRYPT_R, SCRYPT_P, 32)
        .map_err(|e| format!("scrypt params error: {}", e))?;
    let salt = format!("{}{}", website, username);
    let mut scrypt_output = vec![0u8; 32];
    scrypt(sha3_hex.as_bytes(), salt.as_bytes(), &params, &mut scrypt_output)
        .map_err(|e| format!("scrypt error: {}", e))?;

    // Step 3: SHAKE256 PRNG — for call N, feed scrypt_hash (N+1) times, squeeze 256 bits
    // Step 4: Deal/splice index selection + character generation

    // Build pick_index array: [0, 1, 2, ..., length-1]
    let mut pick_index: Vec<usize> = (0..(length as usize)).collect();
    let mut call_counter: u32 = 0;

    // Deal 4 indices for required character classes
    let idx1 = pick_index.remove(gen_index(&scrypt_output, &mut call_counter, pick_index.len()));
    let idx2 = pick_index.remove(gen_index(&scrypt_output, &mut call_counter, pick_index.len()));
    let idx3 = pick_index.remove(gen_index(&scrypt_output, &mut call_counter, pick_index.len()));
    let idx4 = pick_index.remove(gen_index(&scrypt_output, &mut call_counter, pick_index.len()));

    let mut result: Vec<u8> = Vec::with_capacity(length as usize);
    for i in 0..(length as usize) {
        let ch = if i == idx1 {
            pick_char(&scrypt_output, &mut call_counter, LOWER)
        } else if i == idx2 {
            pick_char(&scrypt_output, &mut call_counter, UPPER)
        } else if i == idx3 {
            pick_char(&scrypt_output, &mut call_counter, SYMBOLS)
        } else if i == idx4 {
            pick_char(&scrypt_output, &mut call_counter, NUMBERS)
        } else {
            pick_char(&scrypt_output, &mut call_counter, ALLOWED)
        };
        result.push(ch);
    }

    Ok(String::from_utf8(result).unwrap())
}

/// Generate a deterministic PRNG index for call N: feed scrypt_hash (N+1) times to SHAKE256
fn gen_index(scrypt_hash: &[u8], counter: &mut u32, modulo: usize) -> usize {
    let mut shake = Shake256::default();
    for _ in 0..=(*counter) {
        Update::update(&mut shake, scrypt_hash);
    }
    let mut rng = shake.finalize_xof();
    let mut output = [0u8; 32];
    XofReader::read(&mut rng, &mut output);
    *counter += 1;

    // BigInt modulo: compute (256-bit integer) % modulo
    bigint_modulo(&output, modulo as u64) as usize
}

fn pick_char(scrypt_hash: &[u8], counter: &mut u32, chars: &[u8]) -> u8 {
    let idx = gen_index(scrypt_hash, counter, chars.len());
    chars[idx]
}

/// Compute (big-endian byte array as big integer) % modulo
fn bigint_modulo(bytes: &[u8; 32], modulo: u64) -> u64 {
    let mut remainder: u64 = 0;
    for &byte in bytes.iter() {
        remainder = ((remainder << 8) | (byte as u64)) % modulo;
    }
    remainder
}
