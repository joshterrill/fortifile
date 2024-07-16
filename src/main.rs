use aes::Aes128;
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};
use std::env;
use std::fs::{File, remove_file, metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;
use tempfile::tempdir;

type Aes128Cbc = Cbc<Aes128, Pkcs7>;

const KEY: &[u8; 16] = b"an example key!!"; // 16-byte key for AES128
const IV: &[u8; 16] = b"an example iv!!!";  // 16-byte IV

fn encrypt_file(input_path: &str, output_path: &str, key: &[u8], iv: &[u8]) -> std::io::Result<()> {
    let mut file = File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let cipher = Aes128Cbc::new_from_slices(key, iv).unwrap();
    let encrypted_data = cipher.encrypt_vec(&buffer);

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&encrypted_data)?;
    Ok(())
}

fn decrypt_file(input_path: &str, output_path: &str, key: &[u8], iv: &[u8]) -> std::io::Result<()> {
    let mut file = File::open(input_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let cipher = Aes128Cbc::new_from_slices(key, iv).unwrap();
    let decrypted_data = cipher.decrypt_vec(&buffer).unwrap();

    let mut output_file = File::create(output_path)?;
    output_file.write_all(&decrypted_data)?;
    Ok(())
}

fn log_message(message: &str) {
    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/fortifile.log")
        .unwrap();
    writeln!(log_file, "{}", message).unwrap();
}

fn main() -> notify::Result<()> {
    let args: Vec<String> = env::args().collect();
    log_message(&format!("Received arguments: {:?}", args));

    if args.len() != 2 {
        eprintln!("Usage: {} <path to .enc file>", args[0]);
        log_message("Incorrect number of arguments.");
        return Ok(());
    }

    let encrypted_path = &args[1];
    let temp_dir = tempdir().unwrap();
    let decrypted_path = temp_dir.path().join("test-original.docx");
    let temp_file_path = temp_dir.path().join("~$st-original.docx");

    log_message(&format!("Starting decryption... Encrypted path: {}, Decrypted path: {:?}", encrypted_path, decrypted_path));
    log_message(&format!("Checking permissions for encrypted path: {:?}", metadata(encrypted_path)));

    if let Err(e) = decrypt_file(encrypted_path, decrypted_path.to_str().unwrap(), KEY, IV) {
        eprintln!("Error decrypting file: {}", e);
        log_message(&format!("Error decrypting file: {}", e));
        return Ok(());
    }

    log_message("Decryption successful. Setting up file watcher...");

    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default().with_poll_interval(Duration::from_secs(1)))?;
    watcher.watch(Path::new(decrypted_path.to_str().unwrap()), RecursiveMode::NonRecursive)?;

    log_message("Opening file in default app...");
    opener::open(decrypted_path.to_str().unwrap()).unwrap();

    log_message("Watching for changes...");

    // sleep for 5 seconds
    sleep(Duration::from_secs(5));

    loop {
        if metadata(temp_file_path.to_str().unwrap()).is_err() {
            log_message("Temporary file disappeared, encrypting the updated file...");

            if let Err(e) = encrypt_file(decrypted_path.to_str().unwrap(), encrypted_path, KEY, IV) {
                eprintln!("Error encrypting file: {}", e);
                log_message(&format!("Error encrypting file: {}", e));
            } else {
                if let Err(e) = remove_file(decrypted_path.to_str().unwrap()) {
                    eprintln!("Error deleting file: {}", e);
                    log_message(&format!("Error deleting file: {}", e));
                } else {
                    log_message("File encrypted and original deleted successfully.");
                }
            }
            break;
        }

        if let Ok(event) = rx.try_recv() {
            match event {
                Ok(event) => log_message(&format!("File changed: {:?}", event)),
                Err(e) => log_message(&format!("watch error: {:?}", e)),
            }
        }

        sleep(Duration::from_secs(1));
    }

    Ok(())
}
