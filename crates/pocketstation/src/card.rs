//! The 128 KiB memory card layout shared by PlayStation memory cards and the
//! PocketStation's flash: a directory block followed by 15 data blocks.

pub const CARD_SIZE: usize = 0x20000;
pub const BLOCK_SIZE: usize = 0x2000;
pub const SECTOR_SIZE: usize = 0x80;

/// Directory entry states.
pub const STATE_FIRST: u8 = 0x51;
pub const STATE_MIDDLE: u8 = 0x52;
pub const STATE_LAST: u8 = 0x53;
pub const STATE_FREE: u8 = 0xA0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardFile {
    /// First block number (1..=15), also called the directory index.
    pub block: u8,
    pub name: String,
    pub size: u32,
    /// Blocks in file order.
    pub blocks: Vec<u8>,
}

impl CardFile {
    pub fn is_pocketstation_name(&self) -> bool {
        self.name.as_bytes().get(6) == Some(&b'P')
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardError {
    NotFormatted,
    NameTooLong,
    Exists,
    Full,
    NotFound,
}

impl std::fmt::Display for CardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CardError::NotFormatted => "the card is not formatted",
            CardError::NameTooLong => "file names have at most 20 characters",
            CardError::Exists => "a file with that name already exists",
            CardError::Full => "not enough free blocks on the card",
            CardError::NotFound => "file not found",
        })
    }
}

pub fn checksum(frame: &[u8]) -> u8 {
    frame[..0x7F].iter().fold(0, |a, b| a ^ b)
}

/// A formatted, empty card image.
pub fn formatted() -> Vec<u8> {
    let mut data = vec![0u8; CARD_SIZE];
    data[0] = b'M';
    data[1] = b'C';
    data[0x7F] = checksum(&data[..0x80]);
    for i in 1..16 {
        let frame = &mut data[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE];
        frame[0] = STATE_FREE;
        frame[8] = 0xFF;
        frame[9] = 0xFF;
        frame[0x7F] = checksum(frame);
    }
    for i in 16..36 {
        let frame = &mut data[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE];
        frame[..4].copy_from_slice(&[0xFF; 4]);
        frame[8] = 0xFF;
        frame[9] = 0xFF;
        frame[0x7F] = checksum(frame);
    }
    for i in 36..63 {
        data[i * SECTOR_SIZE..(i + 1) * SECTOR_SIZE].fill(0xFF);
    }
    let header: Vec<u8> = data[..SECTOR_SIZE].to_vec();
    data[63 * SECTOR_SIZE..64 * SECTOR_SIZE].copy_from_slice(&header);
    // Data blocks of a fresh card are erased.
    data[BLOCK_SIZE..].fill(0xFF);
    data
}

pub fn is_formatted(data: &[u8]) -> bool {
    data.len() >= CARD_SIZE && &data[..2] == b"MC"
}

fn frame(data: &[u8], index: usize) -> &[u8] {
    &data[index * SECTOR_SIZE..(index + 1) * SECTOR_SIZE]
}

/// Lists the files on a card image, following each file's block chain.
pub fn files(data: &[u8]) -> Vec<CardFile> {
    let mut out = Vec::new();
    if !is_formatted(data) {
        return out;
    }
    for block in 1..16u8 {
        let entry = frame(data, block as usize);
        if entry[0] != STATE_FIRST {
            continue;
        }
        let size = u32::from_le_bytes(entry[4..8].try_into().unwrap());
        let name_bytes = &entry[0x0A..0x1F];
        let end = name_bytes
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(name_bytes.len());
        let name = String::from_utf8_lossy(&name_bytes[..end]).into_owned();
        let mut blocks = vec![block];
        let mut next = u16::from_le_bytes(entry[8..10].try_into().unwrap());
        while next != 0xFFFF && blocks.len() < 15 {
            let b = next as u8 + 1;
            if !(1..16).contains(&b) || blocks.contains(&b) {
                break;
            }
            blocks.push(b);
            next = u16::from_le_bytes(frame(data, b as usize)[8..10].try_into().unwrap());
        }
        out.push(CardFile {
            block,
            name,
            size,
            blocks,
        });
    }
    out
}

pub fn find(data: &[u8], name: &str) -> Option<CardFile> {
    files(data).into_iter().find(|f| f.name == name)
}

pub fn free_blocks(data: &[u8]) -> Vec<u8> {
    (1..16u8)
        .filter(|b| frame(data, *b as usize)[0] & 0xF0 == 0xA0)
        .collect()
}

/// Reads a file's contents (whole blocks).
pub fn read(data: &[u8], file: &CardFile) -> Vec<u8> {
    let mut out = Vec::with_capacity(file.blocks.len() * BLOCK_SIZE);
    for b in &file.blocks {
        let at = *b as usize * BLOCK_SIZE;
        out.extend_from_slice(&data[at..at + BLOCK_SIZE]);
    }
    out
}

/// Writes a new file, padding the contents to whole blocks. Returns the first block.
pub fn write(data: &mut [u8], name: &str, contents: &[u8]) -> Result<u8, CardError> {
    if !is_formatted(data) {
        return Err(CardError::NotFormatted);
    }
    if name.len() > 20 || name.is_empty() {
        return Err(CardError::NameTooLong);
    }
    if find(data, name).is_some() {
        return Err(CardError::Exists);
    }
    let needed = contents.len().div_ceil(BLOCK_SIZE).max(1);
    let free = free_blocks(data);
    if free.len() < needed {
        return Err(CardError::Full);
    }
    let blocks = &free[..needed];
    for (i, &b) in blocks.iter().enumerate() {
        let at = b as usize * BLOCK_SIZE;
        let chunk_start = i * BLOCK_SIZE;
        let chunk_end = (chunk_start + BLOCK_SIZE).min(contents.len());
        let target = &mut data[at..at + BLOCK_SIZE];
        target.fill(0);
        if chunk_start < contents.len() {
            target[..chunk_end - chunk_start].copy_from_slice(&contents[chunk_start..chunk_end]);
        }
        let entry = &mut data[b as usize * SECTOR_SIZE..(b as usize + 1) * SECTOR_SIZE];
        entry.fill(0);
        entry[0] = if i == 0 {
            STATE_FIRST
        } else if i + 1 == needed {
            STATE_LAST
        } else {
            STATE_MIDDLE
        };
        let next: u16 = blocks.get(i + 1).map(|n| *n as u16 - 1).unwrap_or(0xFFFF);
        entry[8..10].copy_from_slice(&next.to_le_bytes());
        if i == 0 {
            entry[4..8].copy_from_slice(&((needed * BLOCK_SIZE) as u32).to_le_bytes());
            entry[0x0A..0x0A + name.len()].copy_from_slice(name.as_bytes());
        }
        entry[0x7F] = checksum(entry);
    }
    Ok(blocks[0])
}

/// Replaces a file (deleting any previous file with the same name).
pub fn replace(data: &mut [u8], name: &str, contents: &[u8]) -> Result<u8, CardError> {
    if let Some(old) = find(data, name) {
        delete(data, &old);
    }
    write(data, name, contents)
}

/// Marks a file's blocks as deleted.
pub fn delete(data: &mut [u8], file: &CardFile) {
    for &b in &file.blocks {
        let entry = &mut data[b as usize * SECTOR_SIZE..(b as usize + 1) * SECTOR_SIZE];
        entry[0] = match entry[0] {
            STATE_FIRST => 0xA1,
            STATE_MIDDLE => 0xA2,
            STATE_LAST => 0xA3,
            other => other | 0xA0,
        };
        entry[0x7F] = checksum(entry);
    }
}

/// A single save exported with its 128-byte directory entry header (`.mcs`).
pub fn export_single(data: &[u8], file: &CardFile) -> Vec<u8> {
    let mut out = frame(data, file.block as usize).to_vec();
    out[8] = 0xFF;
    out[9] = 0xFF;
    out[0x7F] = checksum(&out);
    out.extend(read(data, file));
    out
}

/// Imports a single-save file: `.mcs` (directory header + data) or a raw
/// PocketStation executable starting with "SC" (named `fallback_name`).
pub fn import_single(data: &mut [u8], file: &[u8], fallback_name: &str) -> Result<u8, CardError> {
    if file.len() > SECTOR_SIZE && file[0] == STATE_FIRST {
        let name_bytes = &file[0x0A..0x1F];
        let end = name_bytes
            .iter()
            .position(|b| *b == 0)
            .unwrap_or(name_bytes.len());
        let name = String::from_utf8_lossy(&name_bytes[..end]).into_owned();
        return replace(data, &name, &file[SECTOR_SIZE..]);
    }
    replace(data, fallback_name, file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_list_read_and_delete() {
        let mut card = formatted();
        assert_eq!(free_blocks(&card).len(), 15);
        let contents: Vec<u8> = (0..BLOCK_SIZE + 10).map(|i| i as u8).collect();
        let block = write(&mut card, "BESLESP00001GAME", &contents).unwrap();
        assert_eq!(block, 1);
        let listed = files(&card);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].blocks, vec![1, 2]);
        assert!(listed[0].is_pocketstation_name());
        let back = read(&card, &listed[0]);
        assert_eq!(&back[..contents.len()], &contents[..]);
        assert_eq!(
            write(&mut card, "BESLESP00001GAME", &[1]),
            Err(CardError::Exists)
        );
        delete(&mut card, &listed[0]);
        assert!(files(&card).is_empty());
        assert_eq!(free_blocks(&card).len(), 15);
        // Every directory frame keeps a valid checksum.
        for i in 0..16 {
            let f = frame(&card, i);
            assert_eq!(checksum(f), f[0x7F]);
        }
    }

    #[test]
    fn single_save_round_trip() {
        let mut card = formatted();
        write(&mut card, "BASLUS-00001SAVE", &[7; 100]).unwrap();
        let file = find(&card, "BASLUS-00001SAVE").unwrap();
        let exported = export_single(&card, &file);
        let mut other = formatted();
        import_single(&mut other, &exported, "unused").unwrap();
        assert_eq!(
            read(&other, &find(&other, "BASLUS-00001SAVE").unwrap())[..100],
            [7; 100]
        );
    }
}
