use std::{
    env,
    fs::{self, File},
    io,
    path::Path,
};

use csv::Writer;
use structures::Record;

pub mod structures;

fn create_csv(path: &Path, wtr: &mut Writer<File>) {
    let file_str = fs::read_to_string(path).unwrap();
    for data in file_str.lines() {
        let record = Record::parse(data);

        // println!("{:#?}", record)
        match record {
            Ok(record) => wtr.serialize(record).unwrap(),
            Err(err) => {}
        }
    }
    wtr.flush().unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let path = Path::new(&args[1]);

    let mut wtr = csv::WriterBuilder::new()
        .has_headers(true)
        .from_path("result.csv")
        .unwrap();
    // .from_writer(io::stdout());

    if path.is_dir() {
        for (idx, entry) in fs::read_dir(path).unwrap().enumerate() {
            print!("processing {idx}, {:#?}", entry);
            match entry {
                Ok(en) => create_csv(&en.path(), &mut wtr),
                Err(_) => unreachable!(),
            }
        }
    } else if path.is_file() {
        create_csv(path, &mut wtr)
    }
}
