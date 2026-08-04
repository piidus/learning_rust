//! # Zip & Custom Binary Wizard
//!
//! An interactive Rust CLI application that compresses (zips) and extracts (unzips)
//! standard `.zip` files as well as custom single-file binary containers (`.bin`).

use dialoguer::{Input, Select};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// 4-byte magic signature header for custom binary containers ("RBIN").
const BIN_MAGIC: &[u8; 4] = b"RBIN";

/// Version indicator for custom binary containers.
const BIN_VERSION: u16 = 1;

/// Entry point of the interactive CLI wizard application.
/// Launches the main selection loop allowing the user to choose compression options.
fn main() {
    println!("=================================================");
    println!("    🧙 ZIP & CUSTOM BINARY WIZARD (RUST) 🧙    ");
    println!("=================================================\n");

    loop {
        let selections = &[
            "📦 Zip a file or folder (.zip)",
            "📂 Unzip a .zip archive",
            "🔒 Pack to Custom Binary (.bin)",
            "🔓 Unpack from Custom Binary (.bin)",
            "❌ Exit",
        ];

        let selection = Select::new()
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&selections[..])
            .interact();

        match selection {
            Ok(0) => {
                if let Err(e) = run_zip_wizard() {
                    println!("\n❌ Error during zipping: {}\n", e);
                }
            }
            Ok(1) => {
                if let Err(e) = run_unzip_wizard() {
                    println!("\n❌ Error during unzipping: {}\n", e);
                }
            }
            Ok(2) => {
                if let Err(e) = run_pack_bin_wizard() {
                    println!("\n❌ Error during binary packing: {}\n", e);
                }
            }
            Ok(3) => {
                if let Err(e) = run_unpack_bin_wizard() {
                    println!("\n❌ Error during binary unpacking: {}\n", e);
                }
            }
            Ok(4) | Err(_) => {
                println!("\nGoodbye! 👋");
                break;
            }
            _ => unreachable!(),
        }
    }
}

// ================= Standard Zip Wizard =================

/// Prompts the user interactively for source and destination paths to create a `.zip` archive.
///
/// # Errors
/// Returns an error if the source path does not exist or if compression fails.
fn run_zip_wizard() -> Result<(), Box<dyn std::error::Error>> {
    let source_str: String = Input::new()
        .with_prompt("Enter file or folder path to ZIP")
        .interact_text()?;

    let source_path = Path::new(&source_str);
    if !source_path.exists() {
        return Err(format!("Source path '{}' does not exist.", source_str).into());
    }

    let default_output = if source_path.is_file() {
        format!("{}.zip", source_path.file_stem().unwrap().to_string_lossy())
    } else {
        format!("{}.zip", source_path.file_name().unwrap().to_string_lossy())
    };

    let output_str: String = Input::new()
        .with_prompt("Enter destination zip path")
        .default(default_output)
        .interact_text()?;

    let output_path = Path::new(&output_str);

    println!("\n⏳ Zipping '{}' -> '{}'...", source_path.display(), output_path.display());

    zip_file_or_dir(source_path, output_path)?;

    println!("✅ Successfully created archive: {}\n", output_path.display());
    Ok(())
}

/// Prompts the user interactively for a `.zip` file path and target folder to extract contents.
///
/// # Errors
/// Returns an error if the `.zip` file is invalid, missing, or extraction fails.
fn run_unzip_wizard() -> Result<(), Box<dyn std::error::Error>> {
    let zip_str: String = Input::new()
        .with_prompt("Enter path to .zip file")
        .interact_text()?;

    let zip_path = Path::new(&zip_str);
    if !zip_path.exists() || !zip_path.is_file() {
        return Err(format!("Zip file '{}' does not exist or is invalid.", zip_str).into());
    }

    let default_output = zip_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "extracted_output".to_string());

    let output_dir_str: String = Input::new()
        .with_prompt("Enter target extraction folder")
        .default(default_output)
        .interact_text()?;

    let output_dir = Path::new(&output_dir_str);

    println!("\n⏳ Unzipping '{}' -> '{}'...", zip_path.display(), output_dir.display());

    unzip_archive(zip_path, output_dir)?;

    println!("✅ Successfully extracted to: {}\n", output_dir.display());
    Ok(())
}

// ================= Custom Binary Wizard =================

/// Prompts the user interactively to pack a file or folder into a custom `.bin` container.
///
/// # Errors
/// Returns an error if the input path does not exist or binary packing fails.
fn run_pack_bin_wizard() -> Result<(), Box<dyn std::error::Error>> {
    let source_str: String = Input::new()
        .with_prompt("Enter file or folder path to PACK to .bin")
        .interact_text()?;

    let source_path = Path::new(&source_str);
    if !source_path.exists() {
        return Err(format!("Source path '{}' does not exist.", source_str).into());
    }

    let default_output = if source_path.is_file() {
        format!("{}.bin", source_path.file_stem().unwrap().to_string_lossy())
    } else {
        format!("{}.bin", source_path.file_name().unwrap().to_string_lossy())
    };

    let output_str: String = Input::new()
        .with_prompt("Enter destination .bin path")
        .default(default_output)
        .interact_text()?;

    let output_path = Path::new(&output_str);

    println!("\n🔒 Packing '{}' -> '{}'...", source_path.display(), output_path.display());

    pack_custom_bin(source_path, output_path)?;

    println!("✅ Successfully created binary container: {}\n", output_path.display());
    Ok(())
}

/// Prompts the user interactively to unpack a custom `.bin` container into a target directory.
///
/// # Errors
/// Returns an error if the `.bin` container header is invalid or extraction fails.
fn run_unpack_bin_wizard() -> Result<(), Box<dyn std::error::Error>> {
    let bin_str: String = Input::new()
        .with_prompt("Enter path to .bin file")
        .interact_text()?;

    let bin_path = Path::new(&bin_str);
    if !bin_path.exists() || !bin_path.is_file() {
        return Err(format!("Binary file '{}' does not exist or is invalid.", bin_str).into());
    }

    let default_output = bin_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unpacked_bin_output".to_string());

    let output_dir_str: String = Input::new()
        .with_prompt("Enter target extraction folder")
        .default(default_output)
        .interact_text()?;

    let output_dir = Path::new(&output_dir_str);

    println!("\n🔓 Unpacking '{}' -> '{}'...", bin_path.display(), output_dir.display());

    unpack_custom_bin(bin_path, output_dir)?;

    println!("✅ Successfully unpacked to: {}\n", output_dir.display());
    Ok(())
}

// ================= ZIP Logic =================

/// Compresses a file or directory recursively into a target `.zip` archive.
///
/// # Arguments
/// * `src` - Path to the input file or folder to compress.
/// * `dst_zip` - Destination path for the generated `.zip` file.
///
/// # Errors
/// Returns an error if file reading, directory walking, or zip writing fails.
pub fn zip_file_or_dir(src: &Path, dst_zip: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let zip_file = File::create(dst_zip)?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated);

    if src.is_file() {
        let filename = src.file_name().ok_or("Invalid file name")?.to_string_lossy();
        zip.start_file(filename, options)?;
        let mut f = File::open(src)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;
    } else if src.is_dir() {
        let parent = src.parent().unwrap_or(Path::new(""));
        let mut buffer = Vec::new();

        for entry in WalkDir::new(src) {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(parent)?;

            let name = relative_path
                .to_str()
                .ok_or("Path contains invalid UTF-8")?
                .replace('\\', "/");

            if name.is_empty() {
                continue;
            }

            if entry.file_type().is_dir() {
                let dir_name = if name.ends_with('/') { name } else { format!("{}/", name) };
                println!("  + Adding folder: {}", dir_name);
                zip.add_directory(dir_name, options)?;
            } else if entry.file_type().is_file() {
                println!("  + Adding file:   {}", name);
                zip.start_file(name, options)?;
                let mut f = File::open(path)?;
                buffer.clear();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }
    }

    zip.finish()?;
    Ok(())
}

/// Extracts a `.zip` archive to the specified destination directory, restoring all directory structures safely.
///
/// # Arguments
/// * `zip_path` - Path to the `.zip` archive to extract.
/// * `dst_dir` - Destination directory path.
///
/// # Errors
/// Returns an error if the zip archive is corrupt or writing files to disk fails.
pub fn unzip_archive(zip_path: &Path, dst_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    if !dst_dir.exists() {
        fs::create_dir_all(dst_dir)?;
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let enclosed_name = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let outpath = dst_dir.join(&enclosed_name);

        if file.name().ends_with('/') || file.is_dir() {
            println!("  - Extracting folder: {}", enclosed_name.display());
            fs::create_dir_all(&outpath)?;
        } else {
            println!("  - Extracting file:   {}", enclosed_name.display());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

// ================= Custom Binary Packing Logic =================

/// Packs a file or directory hierarchy into a custom single-file `.bin` container format.
///
/// Format Header: `RBIN` (4 bytes) + version (2 bytes) + entry count (`u32`).
///
/// # Arguments
/// * `src` - Input file or directory path.
/// * `dst_bin` - Output binary container file path (`.bin`).
///
/// # Errors
/// Returns an error if IO operations fail.
pub fn pack_custom_bin(src: &Path, dst_bin: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut out = File::create(dst_bin)?;

    // 1. Write Header: Magic (4 bytes) + Version (2 bytes)
    out.write_all(BIN_MAGIC)?;
    out.write_all(&BIN_VERSION.to_be_bytes())?;

    // Collect all entries to determine count
    let parent = src.parent().unwrap_or(Path::new(""));
    let mut entries = Vec::new();

    if src.is_file() {
        entries.push(src.to_path_buf());
    } else if src.is_dir() {
        for entry in WalkDir::new(src) {
            let entry = entry?;
            if entry.path() != src {
                entries.push(entry.path().to_path_buf());
            }
        }
    }

    // 2. Write entry count (4 bytes u32)
    out.write_all(&(entries.len() as u32).to_be_bytes())?;

    // 3. Write entries
    let mut buffer = Vec::new();
    for path in entries {
        let relative_path = path.strip_prefix(parent)?;
        let name = relative_path
            .to_str()
            .ok_or("Path contains invalid UTF-8")?
            .replace('\\', "/");

        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len() as u16;

        let is_dir = if path.is_dir() { 1u8 } else { 0u8 };

        // Write filename len + filename string + is_dir flag
        out.write_all(&name_len.to_be_bytes())?;
        out.write_all(name_bytes)?;
        out.write_all(&[is_dir])?;

        if is_dir == 1 {
            println!("  + [BIN] Packing folder: {}", name);
            out.write_all(&0u64.to_be_bytes())?;
        } else {
            println!("  + [BIN] Packing file:   {}", name);
            let mut f = File::open(&path)?;
            buffer.clear();
            f.read_to_end(&mut buffer)?;

            let data_len = buffer.len() as u64;
            out.write_all(&data_len.to_be_bytes())?;
            out.write_all(&buffer)?;
        }
    }

    Ok(())
}

/// Unpacks a custom `.bin` container into the specified target directory.
///
/// # Arguments
/// * `bin_path` - Path to the input `.bin` container.
/// * `dst_dir` - Destination directory where contents will be written.
///
/// # Errors
/// Returns an error if the container magic header is invalid, version is unsupported, or IO operations fail.
pub fn unpack_custom_bin(bin_path: &Path, dst_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut bin = File::open(bin_path)?;

    // 1. Read & Validate Header
    let mut magic = [0u8; 4];
    bin.read_exact(&mut magic)?;
    if &magic != BIN_MAGIC {
        return Err("Invalid binary container magic header.".into());
    }

    let mut ver_buf = [0u8; 2];
    bin.read_exact(&mut ver_buf)?;
    let version = u16::from_be_bytes(ver_buf);
    if version != BIN_VERSION {
        return Err(format!("Unsupported binary container version: {}", version).into());
    }

    // 2. Read Entry Count
    let mut count_buf = [0u8; 4];
    bin.read_exact(&mut count_buf)?;
    let count = u32::from_be_bytes(count_buf);

    if !dst_dir.exists() {
        fs::create_dir_all(dst_dir)?;
    }

    // 3. Unpack Entries
    for _ in 0..count {
        let mut name_len_buf = [0u8; 2];
        bin.read_exact(&mut name_len_buf)?;
        let name_len = u16::from_be_bytes(name_len_buf) as usize;

        let mut name_bytes = vec![0u8; name_len];
        bin.read_exact(&mut name_bytes)?;
        let relative_path_str = String::from_utf8(name_bytes)?;

        let mut is_dir_buf = [0u8; 1];
        bin.read_exact(&mut is_dir_buf)?;
        let is_dir = is_dir_buf[0] == 1;

        let mut data_len_buf = [0u8; 8];
        bin.read_exact(&mut data_len_buf)?;
        let data_len = u64::from_be_bytes(data_len_buf);

        let outpath = dst_dir.join(&relative_path_str);

        if is_dir {
            println!("  - [BIN] Unpacking folder: {}", relative_path_str);
            fs::create_dir_all(&outpath)?;
        } else {
            println!("  - [BIN] Unpacking file:   {}", relative_path_str);
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }

            let mut outfile = File::create(&outpath)?;
            let mut take = (&mut bin).take(data_len);
            io::copy(&mut take, &mut outfile)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_zip_and_unzip_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = Path::new("target/test_tmp_zip");
        if test_dir.exists() {
            fs::remove_dir_all(test_dir)?;
        }
        fs::create_dir_all(test_dir.join("input/nested"))?;
        fs::write(test_dir.join("input/hello.txt"), "Hello Zip World!")?;
        fs::write(test_dir.join("input/nested/sub.txt"), "Subdirectory content")?;

        let zip_path = test_dir.join("test_archive.zip");
        let extract_dir = test_dir.join("output");

        zip_file_or_dir(&test_dir.join("input"), &zip_path)?;
        assert!(zip_path.exists());

        unzip_archive(&zip_path, &extract_dir)?;
        assert!(extract_dir.join("input/hello.txt").exists());
        assert!(extract_dir.join("input/nested/sub.txt").exists());

        let content = fs::read_to_string(extract_dir.join("input/hello.txt"))?;
        assert_eq!(content, "Hello Zip World!");

        fs::remove_dir_all(test_dir)?;
        Ok(())
    }

    #[test]
    fn test_custom_bin_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        let test_dir = Path::new("target/test_tmp_bin");
        if test_dir.exists() {
            fs::remove_dir_all(test_dir)?;
        }
        fs::create_dir_all(test_dir.join("input/subfolder"))?;
        fs::write(test_dir.join("input/data.bin_test"), vec![0xDE, 0xAD, 0xBE, 0xEF])?;
        fs::write(test_dir.join("input/subfolder/info.txt"), "Binary packing test")?;

        let bin_path = test_dir.join("archive.bin");
        let extract_dir = test_dir.join("unpacked");

        // Pack
        pack_custom_bin(&test_dir.join("input"), &bin_path)?;
        assert!(bin_path.exists());

        // Unpack
        unpack_custom_bin(&bin_path, &extract_dir)?;
        assert!(extract_dir.join("input/data.bin_test").exists());
        assert!(extract_dir.join("input/subfolder/info.txt").exists());

        let raw_bytes = fs::read(extract_dir.join("input/data.bin_test"))?;
        assert_eq!(raw_bytes, vec![0xDE, 0xAD, 0xBE, 0xEF]);

        let text_content = fs::read_to_string(extract_dir.join("input/subfolder/info.txt"))?;
        assert_eq!(text_content, "Binary packing test");

        fs::remove_dir_all(test_dir)?;
        Ok(())
    }
}
