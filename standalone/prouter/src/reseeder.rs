use anyhow::{anyhow, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::fs::read;
use std::path::{Path, PathBuf};
use glob::glob;
use rand::seq::SliceRandom;
use std::io::{BufReader, Read, Write};
use byteorder::{BigEndian, WriteBytesExt};


// match SystemTime::now().duration_since(UNIX_EPOCH) {
//     Ok(n) => println!("1970-01-01 00:00:00 UTC was {} seconds ago!", n.as_secs()),
//     Err(_) => panic!("SystemTime before UNIX EPOCH!"),
// }

#[derive(Debug, Clone)]
pub struct SU3File {
    pub signer_id: String,
    signer_id_length: u8,
    signature_type: u16,
    signature_length: u16,
    version_length: u8,
    file_type: u8,
    content_type: u8,
    content: Vec<u8>,
    content_length: u64,
    version: String,
}

impl SU3File {
    pub fn new(signer_id: &str) -> Result<SU3File> {
        Ok(SU3File {
            signer_id: signer_id.to_string(),
            signer_id_length: signer_id.chars().count() as u8,
            signature_type: 6, // 0x0006
            signature_length: 512,
            version_length: 16, // 0x10
            file_type: 0, // 0x00
            content_type: 3, // 0x03
            content: Vec::<u8>::new(),
            content_length: 0,
            version: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string()
        })
    }

    pub fn write(&self, file_path: PathBuf) -> Result<()> {
        let mut file = fs::File::create(file_path)?;
        file.write_all("I2Psu3".as_bytes())?;
        file.write_all(&[0, 0])?;
        file.write_u16::<BigEndian>(self.signature_type)?;
        file.write_u16::<BigEndian>(self.signature_length)?;
        file.write_all(&[0])?;
        file.write_u8(self.version_length)?;
        file.write_all(&[0])?;
        file.write_u8(self.signer_id_length)?;
        file.write_u64::<BigEndian>(self.content_length)?;
        file.write_all(&[0])?;
        file.write_u8(self.file_type)?;
        file.write_all(&[0])?;
        file.write_u8(self.content_type)?;
        for _ in 0..12 {
            file.write_all(&[0])?;
        }
        let version_len = self.version.chars().count();
        file.write_all(self.version.as_bytes())?;
        for _ in 0..(16 - version_len) {
            file.write_all(&[0])?;
        }
        file.write_all(self.signer_id.as_bytes())?;
        file.write_all(&self.content[..])?;

        Ok(())
    }

    pub fn reseed(&mut self, netdb: &str) -> Result<()> {
        let seed = 75;
        let mut dat_files: Vec<PathBuf> = vec![];
        for entry in glob(format!("{}/**/*.dat", netdb).as_str())? {
            let entry = entry?;
            let path = entry.to_path_buf();
            let metadata = entry.metadata()?;
            let last_modified = metadata.modified()?.elapsed()?.as_secs();
            if last_modified > 10 * 3600 && metadata.is_file() {
                dat_files.push(path);
            }
        }

        if dat_files.len() == 0 {
            return Err(anyhow!("Can't get enough netDb entries"));
        } else if dat_files.len() > seed {
            let mut rng = &mut rand::thread_rng();
            dat_files = dat_files.choose_multiple(&mut rng, seed).cloned().collect();
        }

        let mut read_buffer = Vec::new();
        let mut write_buffer = Vec::new();
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(write_buffer));

        for file_path in &dat_files {
            let file_name = file_path.file_name().ok_or(anyhow!("Can't fetch netDb filenames"))?;
            let mut file = fs::File::open(&file_path)?;
            file.read_to_end(&mut read_buffer);
            zip.start_file(file_name.to_str().ok_or(anyhow!("Can't recognize netDb filenames"))?, zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored))?;
            zip.write_all(&*read_buffer)?;
            read_buffer.clear();
        }
        let writer = zip.finish()?;

        self.file_type = 0;
        self.content_type = 3;
        self.content = writer.get_ref().clone();
        self.content_length = self.content.len() as u64;

        Ok(())
    }
}