use std::{env, fs, process};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use minisign_verify::{PublicKey, Signature};

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let artifact_path = args
        .next()
        .ok_or_else(|| "missing updater artifact path".to_string())?;
    let signature_path = args
        .next()
        .ok_or_else(|| "missing updater signature path".to_string())?;
    if args.next().is_some() {
        return Err("unexpected verifier argument".to_string());
    }

    let encoded_public_key = env::var("TAURI_UPDATER_PUBLIC_KEY")
        .map_err(|_| "TAURI_UPDATER_PUBLIC_KEY is not configured".to_string())?;
    let public_key_document = STANDARD
        .decode(encoded_public_key.trim())
        .map_err(|error| format!("invalid updater public key encoding: {error}"))?;
    let public_key_document = std::str::from_utf8(&public_key_document)
        .map_err(|error| format!("invalid updater public key document: {error}"))?;
    let public_key = PublicKey::decode(public_key_document)
        .map_err(|error| format!("invalid updater public key: {error}"))?;

    let encoded_signature = fs::read_to_string(&signature_path)
        .map_err(|error| format!("unable to read updater signature: {error}"))?;
    let signature_document = STANDARD
        .decode(encoded_signature.trim())
        .map_err(|error| format!("invalid updater signature encoding: {error}"))?;
    let signature_document = std::str::from_utf8(&signature_document)
        .map_err(|error| format!("invalid updater signature document: {error}"))?;
    let signature = Signature::decode(signature_document)
        .map_err(|error| format!("invalid updater signature: {error}"))?;

    let artifact = fs::read(&artifact_path)
        .map_err(|error| format!("unable to read updater artifact: {error}"))?;
    public_key
        .verify(&artifact, &signature, true)
        .map_err(|error| format!("updater signature mismatch: {error}"))?;

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        process::exit(1);
    }
}
