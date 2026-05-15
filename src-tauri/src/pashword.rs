use sha3::{Digest, Sha3_512, Shake256};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use scrypt::{scrypt, Params};

const SCRYPT_N: u8 = 14;       // log2(16384) = 14
const SCRYPT_R: u32 = 8;
const SCRYPT_P: u32 = 1;

const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const DIGITS: &[u8] = b"0123456789";
const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

pub fn generate_pashword(
    website: &str,
    username: &str,
    secret_key: &str,
    length: u8,
) -> Result<String, String> {
    let input = format!("{}{}{}{}", website, username, secret_key, length);

    // Step 1: SHA3-512
    let sha3_hash = {
        let mut hasher = Sha3_512::new();
        Update::update(&mut hasher, input.as_bytes());
        hasher.finalize()
    };

    // Step 2: scrypt (N=16384, r=8, p=1)
    let params = Params::new(SCRYPT_N, SCRYPT_R, SCRYPT_P, 64)
        .map_err(|e| format!("scrypt params error: {}", e))?;
    let salt = &sha3_hash[..32];
    let mut scrypt_output = vec![0u8; 64];
    scrypt(&sha3_hash, salt, &params, &mut scrypt_output)
        .map_err(|e| format!("scrypt error: {}", e))?;

    // Step 3: SHAKE-256 as CSPRNG
    let mut shake = Shake256::default();
    Update::update(&mut shake, &scrypt_output);
    let mut rng = shake.finalize_xof();

    // Step 4: Generate password with required character classes
    let mut chars: Vec<u8> = Vec::with_capacity(length as usize);
    let mut buf = [0u8; 1];

    // Ensure at least one of each required class
    let sets: &[&[u8]] = &[UPPER, LOWER, DIGITS, SYMBOLS];
    for set in sets {
        XofReader::read(&mut rng, &mut buf);
        chars.push(set[buf[0] as usize % set.len()]);
    }

    let all: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?";
    while chars.len() < length as usize {
        XofReader::read(&mut rng, &mut buf);
        chars.push(all[buf[0] as usize % all.len()]);
    }

    // Shuffle using Fisher-Yates with CSPRNG
    for i in (1..chars.len()).rev() {
        XofReader::read(&mut rng, &mut buf);
        let j = buf[0] as usize % (i + 1);
        chars.swap(i, j);
    }

    Ok(String::from_utf8(chars).unwrap())
}

/// Derive an AES-256 key from master password using same pipeline.
/// Returns 32 bytes suitable for AES-256-GCM.
pub fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let input = format!("{}{}32", master_password, hex::encode(salt));

    let sha3_hash = {
        let mut hasher = Sha3_512::new();
        Update::update(&mut hasher, input.as_bytes());
        hasher.finalize()
    };

    let params = Params::new(SCRYPT_N, SCRYPT_R, SCRYPT_P, 64)
        .map_err(|e| format!("scrypt params error: {}", e))?;
    let scrypt_salt = &sha3_hash[..32];
    let mut scrypt_output = vec![0u8; 64];
    scrypt(&sha3_hash, scrypt_salt, &params, &mut scrypt_output)
        .map_err(|e| format!("scrypt error: {}", e))?;

    let mut shake = Shake256::default();
    Update::update(&mut shake, &scrypt_output);
    let mut rng = shake.finalize_xof();
    let mut key = [0u8; 32];
    XofReader::read(&mut rng, &mut key);

    Ok(key)
}
