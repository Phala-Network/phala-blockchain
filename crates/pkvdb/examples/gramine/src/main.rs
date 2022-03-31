use rusty_leveldb;
use std::path::Path;
use std::rc::Rc;
use std::fs::File;
use std::io::Read;
use std::io::Write;

const DB_PATH: &'static str = "/data";
const ALLOWED_PATH: &'static str = "/allowed";

fn aux_get_byte_slice<T: AsRef<[u8]>>(source: &'_ T) -> &'_ [u8] {
    source.as_ref()
}

fn main() {
    let path = format!("{}/simple_db_workds", DB_PATH);
    {
        let mut options = rusty_leveldb::Options::default();
        options.env = Rc::new(Box::new(rusty_leveldb::gramine_env::GramineEnv::new()));
        let mut db = rusty_leveldb::DB::open(Path::new(&path), options).unwrap();
        assert!(db.put(b"k1", b"v1").is_ok());
        let result = db.get(b"k1").unwrap();
        assert_eq!(aux_get_byte_slice(&result), b"v1");
        db.flush(); 
    }

    {
        let manifest_file = format!("{}/simple_db_workds/MANIFEST-000001", DB_PATH);
        let mut manifest_handler = std::fs::OpenOptions::new().read(true).open(&manifest_file).unwrap();
        let mut buffer = Vec::new();
        let _  = manifest_handler.read_to_end(&mut buffer);
        println!("result {:?}", &buffer);
        let allowed_file = format!("{}/MANIFEST", ALLOWED_PATH);
        let mut allowed_handler = std::fs::OpenOptions::new().create(true).write(true).open(&allowed_file).unwrap();
        let _ = allowed_handler.write_all(&buffer);
        allowed_handler.flush();
    }
}
