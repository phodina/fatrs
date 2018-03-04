extern crate byteorder;

use byteorder::{ByteOrder, LittleEndian};
use std::io::{Read, SeekFrom};
use std::fs::File;
use std::path::PathBuf;
use std::io::prelude::*;

struct PartitionEntry {
    first_byte: u8,
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
