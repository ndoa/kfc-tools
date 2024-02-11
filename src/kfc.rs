use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, SeekFrom};

#[derive(Debug)]
#[allow(unused)]
pub struct KFCDirFileHeader {
    pub magic: u32,
    pub count: u32,
    pub count2: u32,
    pub field_0xc: u32,
    pub data_file_size: u64,
}

fn read_kfc_file_header<T>(rdr: &mut T) -> Result<KFCDirFileHeader>
where
    T: Read + std::io::Seek,
{
    let header = KFCDirFileHeader {
        magic: rdr.read_u32::<LittleEndian>()?,
        count: rdr.read_u32::<LittleEndian>()?,
        count2: rdr.read_u32::<LittleEndian>()?,
        field_0xc: rdr.read_u32::<LittleEndian>()?,
        data_file_size: rdr.read_u64::<LittleEndian>()?,
    };

    Ok(header)
}

#[derive(Debug)]
#[allow(unused)]
pub struct KFCSizeEntry {
    pub _offset: u64,
    pub size_0: u32,
    pub size_1: u32,
    pub entry_index: u32,
    pub unk: u32,
}

fn read_kfc_size_entry<T>(rdr: &mut T) -> Result<KFCSizeEntry>
where
    T: ReadBytesExt + std::io::Seek,
{
    Ok(KFCSizeEntry {
        _offset: rdr.stream_position()?,
        size_0: rdr.read_u32::<LittleEndian>()?,
        size_1: rdr.read_u32::<LittleEndian>()?,
        entry_index: rdr.read_u32::<LittleEndian>()?,
        unk: rdr.read_u32::<LittleEndian>()?,
    })
}

#[derive(Debug)]
#[allow(unused)]
pub struct KFCDirFile {
    pub _offset: u64,
    pub header: KFCDirFileHeader,
    pub _data_start: u64,

    pub hash_table: Vec<u64>,
    pub size_table: Vec<KFCSizeEntry>,
    pub offset_table: Vec<u64>,
}

pub fn read_kfc_dir_file<T>(rdr: &mut T) -> Result<KFCDirFile>
where
    T: Read + ReadBytesExt + std::io::Seek,
{
    let _offset = rdr.stream_position()?;
    let header = read_kfc_file_header(rdr)?;
    let _data_start = rdr.stream_position()?;

    // I've never seen a case where these are different, but the offset table
    // explicitly uses `.count2` instead of `.count`.
    assert_eq!(header.count, header.count2);

    // hash table (file "names")
    _ = rdr.seek(SeekFrom::Start(_data_start));
    let mut hash_table: Vec<u64> = Vec::new();
    for _ in 0..header.count {
        let entry = rdr.read_u64::<LittleEndian>()?;
        hash_table.push(entry);
    }

    // Size Table
    _ = rdr.seek(SeekFrom::Start(_data_start + 8 * header.count as u64));
    let mut size_table: Vec<KFCSizeEntry> = Vec::new();
    for _ in 0..header.count {
        let entry = read_kfc_size_entry(rdr)?;
        size_table.push(entry);
    }

    // Offset Table
    _ = rdr.seek(SeekFrom::Start(_data_start + 24 * header.count2 as u64));
    let mut offset_table: Vec<u64> = Vec::new();
    for _ in 0..header.count2 {
        let entry = rdr.read_u64::<LittleEndian>()?;
        offset_table.push(entry);
    }

    Ok(KFCDirFile {
        _offset,
        header,
        _data_start,
        hash_table,
        size_table,
        offset_table,
    })
}
