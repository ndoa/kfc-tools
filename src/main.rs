use anyhow::Result;
use std::{
    env,
    fs::{self, File},
    io::{BufReader, Read, Seek, SeekFrom, Write},
};
mod kfc;
use kfc::read_kfc_dir;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!(
            "Usage: kfc_extractor <path to enshrouded.kfc_dir> <path to enshrouded.kfc_data>"
        );
        anyhow::bail!("Not enough arguments");
    }

    let dir_filepath = &args[1];
    let data_filepath = &args[2];

    println!("Attempting to parse kfc dir:{dir_filepath}");

    // Read kfc_dir index file.
    let mut dir_file = BufReader::new(File::open(dir_filepath)?);
    let kfc_dir = read_kfc_dir(&mut dir_file)?;
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

        // Write to output

        // CRPF magic header for resource packages
        let category = {
            if file_data[0] == 0x43
                && file_data[1] == 0x52
                && file_data[2] == 0x50
                && file_data[3] == 0x46
            {
                "resource_packages"
            } else {
                "raw"
            }
        };
        let output_dir = format!("./output/{}", category);
        let output_filepath = format!("{}/{:X}", output_dir, file_name_hash);
        fs::create_dir_all(&output_dir)?;

        let mut out_file = File::create(&output_filepath)?;
        out_file.write_all(&file_data)?;
    }

    return Ok(());
}
