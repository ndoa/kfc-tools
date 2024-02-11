use anyhow::{Ok, Result};
use clap::error::ErrorKind;
use clap::{CommandFactory, Parser, Subcommand};
use ksc::read_ksc1_file;
use std::io::Cursor;
use std::path::PathBuf;
use std::{
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom, Write},
};

mod hash;
mod kfc;
mod ksc;
use kfc::read_kfc_dir_file;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    ExtractKFC {
        /// Path to the ".kfc_dir" file
        kfc_dir_path: PathBuf,

        /// Path to the ".kfc_data" file
        kfc_data_path: PathBuf,
    },
    ExtractKSC1 {
        /// Path to the KSC1 file
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.cmd {
        Commands::ExtractKFC {
            kfc_dir_path,
            kfc_data_path,
        } => {
            let mut cmd = Args::command();
            if !kfc_dir_path.exists() {
                cmd.error(
                    ErrorKind::ValueValidation,
                    format!(
                        "kfc_dir_path `{}` doesn't exist or is inaccessible",
                        kfc_dir_path.display()
                    ),
                )
                .exit()
            }

            if !kfc_data_path.exists() {
                cmd.error(
                    ErrorKind::ValueValidation,
                    format!(
                        "kfc_data_path `{}` doesn't exist or is inaccessible",
                        kfc_data_path.display()
                    ),
                )
                .exit()
            }

            return extract_kfc(
                kfc_dir_path.to_str().unwrap(),
                kfc_data_path.to_str().unwrap(),
            );
        }
        Commands::ExtractKSC1 { path } => {
            let mut cmd = Args::command();
            if !path.exists() {
                cmd.error(
                    ErrorKind::ValueValidation,
                    format!("path `{}` doesn't exist or is inaccessible", path.display()),
                )
                .exit()
            }
            return extract_ksc(path.to_str().unwrap());
        }
    }
}

/// Extracts a kfc_dir+kfc_data file.
fn extract_kfc(dir_filepath: &str, data_filepath: &str) -> Result<()> {
    println!("Attempting to parse kfc dir:{dir_filepath}");

    // Read kfc_dir index file.
    let mut dir_file = BufReader::new(File::open(dir_filepath)?);
    let kfc_dir = read_kfc_dir_file(&mut dir_file)?;
    println!("Loaded KFC dir! Hash count: {}", kfc_dir.hash_table.len());

    // Extract
    let mut data_file = BufReader::new(File::open(data_filepath)?);

    for i in 0..kfc_dir.header.count {
        let file_name_hash: u64 = kfc_dir.hash_table[i as usize];
        let file_offset = kfc_dir.offset_table[i as usize];
        let file_size = kfc_dir.size_table[i as usize].size_0;
        println!(
            "Extracting entry (index:{}, hash: {:X}, offset: {:X}, size:{:X})",
            i, file_name_hash, file_offset, file_size
        );

        // Read from data file
        let _ = data_file.seek(SeekFrom::Start(file_offset));
        let mut file_data: Vec<u8> = vec![0; (file_size).try_into()?];
        data_file.read_exact(&mut file_data)?;

        // Split into raw or resource_packages folder based on
        // CRPF magic header for resource packages
        let category = {
            if file_data.len() > 4
                && file_data[0] == 0x43
                && file_data[1] == 0x52
                && file_data[2] == 0x50
                && file_data[3] == 0x46
            {
                "resource_packages"
            } else {
                "raw"
            }
        };

        // Ensure output file directory tree exists
        let output_dir = format!("./output/kfc/{}", category);
        let output_filepath = format!("{}/{:X}", output_dir, file_name_hash);
        fs::create_dir_all(&output_dir)?;

        // Write to disk
        let mut out_file = File::create(&output_filepath)?;
        out_file.write_all(&file_data)?;
    }

    Ok(())
}

/// Extracts a KSC1 file
fn extract_ksc(path: &str) -> Result<()> {
    let mut file = BufReader::new(File::open(path)?);
    let ksc_file = read_ksc1_file(&mut file)?;

    for i in 0..ksc_file.header.section_count {
        let toc_entry = &ksc_file.toc[i as usize];
        let section_name = String::from_utf8(toc_entry.name_bytes.clone())?;

        // Ensure output file directory tree exists
        let output_dir = "./output/ksc";
        let output_filepath = format!("{}/{}-{}-{:X}", output_dir, i, section_name, toc_entry.hash);
        fs::create_dir_all(output_dir)?;

        // Decompress
        let compressed_reader = Cursor::new(&ksc_file.sections[i as usize]);
        let mut decompressed_reader = zstd::stream::Decoder::new(compressed_reader)?;

        // Write to disk
        let mut out_file = File::create(&output_filepath)?;
        std::io::copy(&mut decompressed_reader, &mut out_file)?;
    }

    Ok(())
}
