extern crate byteorder;

use byteorder::{ByteOrder, LittleEndian};
use std::io::{Read, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;

struct FATEntry {
    filename: String,
    ext: String,
    attributes: u8,
    modify_time: u16,
    modify_date: u16,
    starting_cluster: u16,
    file_size: u32,
}

impl FATEntry {
    fn new(data: &[u8]) -> Option<FATEntry> {
        match data[0] {
            // Unused entry
            0x00 => return None,
            // Deleted entry
            0x45 => (),
            // File starting with 0xe5
            0x05 => (),
            // Directory
            0x2e => (),
            _ => (),
        }

        let filename = String::from_utf8(data[0..8].to_vec()).unwrap();
        let ext = String::from_utf8(data[8..11].to_vec()).unwrap();

        Some(FATEntry {
            filename: filename,
            ext: ext,
            attributes: data[11],
            modify_time: LittleEndian::read_u16(&data[21..23]),
            modify_date: LittleEndian::read_u16(&data[23..25]),
            starting_cluster: LittleEndian::read_u16(&data[25..27]),
            file_size: LittleEndian::read_u32(&data[27..31]),
        })
    }

    fn display(&self) {
        println!("Filename: {} Extension: {}", self.filename, self.ext);
        println!("Modified: {} {}", self.modify_time, self.modify_date);
        println!("Start: {} Size: {}", self.starting_cluster, self.file_size);
    }
}

struct BootSector {
    jmp: [u8; 3],
    oem: String,
    sector_size: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    number_of_fats: u8,
    root_dir_entries: u16,
    total_sectors_short: u16,
    media_descriptor: u8,
    fat_size_sectors: u16,
    sectors_per_track: u16,
    number_of_heads: u16,
    hidden_sectors: u32,
    total_sectors_long: u32,

    drive_number: u8,
    current_head: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fs_type: [u8; 8],
    boot_sector_signature: u16,
}

impl BootSector {
    fn new(data: &[u8]) -> BootSector {
        let mut volume_label = [0u8; 11];
        volume_label.copy_from_slice(&data[43..54]);

        let mut fs_type = [0u8; 8];
        fs_type.copy_from_slice(&data[54..62]);

        let oem = String::from_utf8(data[3..11].to_vec()).unwrap();

        BootSector {
            jmp: [data[2], data[1], data[0]],
            oem: oem,
            sector_size: LittleEndian::read_u16(&data[11..13]),
            sectors_per_cluster: data[13],
            reserved_sectors: LittleEndian::read_u16(&data[14..16]),
            number_of_fats: data[16],
            root_dir_entries: LittleEndian::read_u16(&data[17..19]),
            total_sectors_short: LittleEndian::read_u16(&data[19..21]),
            media_descriptor: data[21],
            fat_size_sectors: LittleEndian::read_u16(&data[22..24]),
            sectors_per_track: LittleEndian::read_u16(&data[24..26]),
            number_of_heads: LittleEndian::read_u16(&data[26..28]),
            hidden_sectors: LittleEndian::read_u32(&data[28..32]),
            total_sectors_long: LittleEndian::read_u32(&data[32..36]),

            drive_number: data[36],
            current_head: data[37],
            boot_signature: data[38],
            volume_id: LittleEndian::read_u32(&data[39..43]),
            volume_label: volume_label,
            fs_type: fs_type,
            boot_sector_signature: LittleEndian::read_u16(&data[510..512]),
        }
    }

    fn display(&self) {
        println!(
            "Jump code: 0x{:x}:0x{:x}:0x{:x}",
            self.jmp[2], self.jmp[1], self.jmp[0]
        );
        println!("OEM: {}", self.oem);
        println!("Sector size: {}", self.sector_size);
        println!("Sectors per cluster: {}", self.sectors_per_cluster);
        println!("Reserved sectors: {}", self.reserved_sectors);
        println!("Number of FATs: {}", self.number_of_fats);
        println!("Root dir entries: {}", self.root_dir_entries);
        println!("Total sectors short: {}", self.total_sectors_short);
        println!("Media descriptor: 0x{:x}", self.media_descriptor);
        println!("FAT size sectors: {}", self.fat_size_sectors);
        println!("Sectors per track: {}", self.sectors_per_track);
        println!("Number of heads: {}", self.number_of_heads);
        println!("Hidden sectors: {}", self.hidden_sectors);
        println!("Total sectors long: {}", self.total_sectors_long);
        println!("Drive number: 0x{:x}", self.drive_number);
        println!("Current head: 0x{:x}", self.current_head);
        println!("Boot signature: 0x{:x}", self.boot_signature);
        println!("Volume id: 0x{:x}", self.volume_id);
        println!("Volume label: {:?}", self.volume_label);
        println!("Filesystem type: {:?}", self.fs_type);
        println!("Bootsector signature: 0x{:x}", self.boot_sector_signature);
    }
}

struct PartitionEntry {
    pub first_byte: u8,
    start_chs: [u8; 3],
    partition_type: u8,
    end_chs: [u8; 3],
    start_sector: u32,
    length_sectors: u32,
}

impl PartitionEntry {
    fn new(data: &[u8]) -> PartitionEntry {
        PartitionEntry {
            first_byte: data[0],
            start_chs: [data[1], data[2], data[3]],
            partition_type: data[4],
            end_chs: [data[5], data[6], data[7]],
            start_sector: LittleEndian::read_u32(&data[8..12]),
            length_sectors: LittleEndian::read_u32(&data[12..16]),
        }
    }

    fn display(&self) {
        println!("First byte: 0x{}", self.first_byte);
        println!("Type: {}", self.partition_type);
        println!(
            "Start in CHS: 0x{:x}:0x{:x}:0x{:x}",
            self.start_chs[2], self.start_chs[1], self.start_chs[0]
        );
        println!(
            "End in CHS: 0x{:x}:0x{:x}:0x{:x}",
            self.end_chs[2], self.end_chs[1], self.end_chs[0]
        );

        println!(
            "Relative LBA address 0x{:08x}, {} sectors long",
            self.start_sector, self.length_sectors,
        );
    }
}

fn open_fatfs(img: PathBuf) {
    let mut file = File::open(img).unwrap();

    file.seek(SeekFrom::Start(0x1BE)).unwrap();

    for i in 1..4 {
        println!("Partition #{}", i);

        let mut buffer = [0u8; 16];

        file.read(&mut buffer).unwrap();

        let partition = PartitionEntry::new(&buffer);
        partition.display();

        if partition.partition_type == 0x6 {
            println!("Found partition. Seek to first sector...");

            file.seek(SeekFrom::Start(512u64 * partition.start_sector as u64))
                .unwrap();

            let mut buffer = [0u8; 512];
            file.read(&mut buffer).unwrap();

            let boot_sector = BootSector::new(&buffer);
            boot_sector.display();

            println!("Seek to root dir...");
            let skip = (boot_sector.reserved_sectors as i64 - 1
                + boot_sector.fat_size_sectors as i64 * boot_sector.number_of_fats as i64)
                * boot_sector.sector_size as i64;
            file.seek(SeekFrom::Current(skip)).unwrap();

            for j in 0..boot_sector.root_dir_entries {
                let mut buffer = [0u8; 32];
                file.read(&mut buffer).unwrap();

                let entry = FATEntry::new(&buffer);
                match entry {
                    Some(entry) => entry.display(),
                    None => (),
                }
            }

            break;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        open_fatfs(PathBuf::from("samples/test.img"));
    }
}
