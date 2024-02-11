use std::io::Read;

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};

const KSC1_FILE_MAGIC: u32 = 0x3143534B; // b"KSC1"

#[derive(Debug)]
#[allow(unused)]
pub struct KSC1Header {
    pub _offset: u64,
    pub magic: u32,
    pub section_count: u32,
    pub header_crc: u64,
    pub unk: u64,
}

fn read_ksc1_header<T>(rdr: &mut T) -> Result<KSC1Header>
where
    T: ReadBytesExt + std::io::Seek,
{
    Ok(KSC1Header {
        _offset: rdr.stream_position()?,
        magic: rdr.read_u32::<LittleEndian>()?,
        section_count: rdr.read_u32::<LittleEndian>()?,
        header_crc: rdr.read_u64::<LittleEndian>()?,
        unk: rdr.read_u64::<LittleEndian>()?,
    })
}

#[derive(Debug)]
#[allow(unused)]
pub struct KSC1TOCEntry {
    pub _offset: u64,
    pub hash: u32,
    pub name_bytes: Vec<u8>, // 4 bytes
    pub size: u32,
}

fn read_ksc1_toc_entry<T>(rdr: &mut T) -> Result<KSC1TOCEntry>
where
    T: ReadBytesExt + std::io::Seek,
{
    let hash = rdr.read_u32::<LittleEndian>()?;

    let mut name_bytes = vec![0u8; 4];
    rdr.read_exact(&mut name_bytes)?;

    let size = rdr.read_u32::<LittleEndian>()?;

    Ok(KSC1TOCEntry {
        _offset: rdr.stream_position()?,
        hash,
        name_bytes,
        size,
    })
}

#[derive(Debug)]
#[allow(unused)]
pub struct KSC1File {
    pub _offset: u64,
    pub header: KSC1Header,

    _toc_start: u64,
    pub toc: Vec<KSC1TOCEntry>,

    _sections_start: u64,
    pub sections: Vec<Vec<u8>>,
}

pub fn read_ksc1_file<T>(rdr: &mut T) -> Result<KSC1File>
where
    T: Read + ReadBytesExt + std::io::Seek,
{
    let _offset = rdr.stream_position()?;
    let header = read_ksc1_header(rdr)?;
    assert_eq!(KSC1_FILE_MAGIC, header.magic);

    // TOC
    let _toc_start = rdr.stream_position()?;
    let mut toc: Vec<KSC1TOCEntry> = Vec::new();
    for _ in 0..header.section_count {
        let entry = read_ksc1_toc_entry(rdr)?;
        toc.push(entry);
    }

    // Sections
    let _sections_start = rdr.stream_position()?;

    let mut sections: Vec<Vec<u8>> = Vec::new();
    for i in 0..header.section_count {
        let section_size = toc[i as usize].size;
        let mut section_bytes = vec![0u8; section_size.try_into()?];
        rdr.read_exact(&mut section_bytes)?;
        sections.push(section_bytes);
    }

    Ok(KSC1File {
        _offset,
        header,
        _toc_start,
        toc,
        _sections_start,
        sections,
    })
}
