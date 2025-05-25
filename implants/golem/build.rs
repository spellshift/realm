use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use walkdir::WalkDir;
// include_dir_codegen is the typical way include_dir is used in build scripts.
// The include_dir crate itself is usually a runtime dependency.
use include_dir_codegen;

const DEFAULT_GOLEM_ENC_KEY: &str = "what is that sound I here?";
const DEFAULT_GOLEM_KDF_SALT: &str = "the far realms call to me";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=GOLEM_ENC_KEY");
    println!("cargo:rerun-if-env-changed=GOLEM_KDF_SALT");

    // Handle cfg(target_os = "windows")
    if env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        static_vcruntime::metabuild();
    }

    // Read Environment Variables
    let golem_enc_key = env::var("GOLEM_ENC_KEY")
        .unwrap_or_else(|_| DEFAULT_GOLEM_ENC_KEY.to_string());
    let golem_kdf_salt = env::var("GOLEM_KDF_SALT")
        .unwrap_or_else(|_| DEFAULT_GOLEM_KDF_SALT.to_string());

    // Key Derivation (PBKDF2HMAC-SHA256)
    let mut derived_key_material = [0u8; 44]; // 32 for AES key, 12 for AES nonce
    pbkdf2_hmac::<Sha256>(
        golem_enc_key.as_bytes(),
        golem_kdf_salt.as_bytes(),
        10_000, // Iterations
        &mut derived_key_material,
    );

    let aes_key_bytes: [u8; 32] = derived_key_material[0..32].try_into().unwrap();
    let aes_nonce_bytes: [u8; 12] = derived_key_material[32..44].try_into().unwrap();

    // Store Derived Crypto Parameters
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crypto_config_path = out_dir.join("crypto_config.rs");

    let crypto_config_content = format!(
        "pub const AES_KEY: &[u8; 32] = &{:?};\npub const AES_NONCE: &[u8; 12] = &{:?};\npub const KDF_SALT: &str = {:?};\n",
        aes_key_bytes,
        aes_nonce_bytes,
        golem_kdf_salt
    );
    fs::write(&crypto_config_path, crypto_config_content).unwrap();

    // Locate and Encrypt Tomes
    let cargo_manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_tomes_dir = cargo_manifest_dir.join("embed_files_golem_prod");
    let encrypted_tomes_dir = out_dir.join("encrypted_tomes");

    if !source_tomes_dir.exists() {
        println!("cargo:warning=Source tomes directory {} does not exist. Skipping encryption.", source_tomes_dir.display());
        // Still create the encrypted_tomes_dir for include_dir to work, even if it's empty.
        fs::create_dir_all(&encrypted_tomes_dir).unwrap();
    } else {
        fs::create_dir_all(&encrypted_tomes_dir).unwrap();
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&aes_key_bytes));

        for entry in WalkDir::new(&source_tomes_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name().to_string_lossy() == "main.eldritch" && entry.file_type().is_file() {
                let plaintext = fs::read(entry.path()).unwrap();
                let nonce = Nonce::from_slice(&aes_nonce_bytes); // Use the derived nonce
                let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).expect("encryption failed!");

                // Create the corresponding directory structure in $OUT_DIR/encrypted_tomes/
                let relative_path = entry.path().strip_prefix(&source_tomes_dir).unwrap();
                let target_path = encrypted_tomes_dir.join(relative_path);
                
                if let Some(parent_dir) = target_path.parent() {
                    fs::create_dir_all(parent_dir).unwrap();
                }
                fs::write(&target_path, ciphertext).unwrap();
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
        }
    }
    
    // Embed Encrypted Tomes using include_dir
    // The path provided to embed must be relative to CARGO_MANIFEST_DIR or absolute.
    // $OUT_DIR is an absolute path.
    let options = include_dir_codegen::EmbedOptions::default().glob("**/*.eldritch");
    include_dir_codegen::embed("GOLEM_ENCRYPTED_TOMES", &encrypted_tomes_dir, options)
        .expect("Failed to embed encrypted tomes");

}
