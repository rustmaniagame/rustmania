use std::{
    env::current_dir,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

fn main() {
    if let Ok(path) = current_dir() {
        println!("Paths are relative to {:?}.\n", path);
    } else {
        println!("Could not determine program location.\n")
    }
    let mut file_path = String::new();
    print!("Enter simfile path: ");
    let _ = io::stdout().flush();
    io::stdin()
        .read_line(&mut file_path)
        .expect("failed to read input");
    let mut file_path = PathBuf::from(file_path.trim_end_matches('\n'));
    let notedata = if let Some(extension) = file_path.extension() {
        let sim = File::open(file_path.as_path()).expect("");
        match extension.to_str() {
            Some("sm") => {
                notedata::NoteData::from_sm_reader(sim).expect("Could not deserialize .sm")
            }
            Some("dwi") => {
                notedata::NoteData::from_dwi_reader(sim).expect("Could note deseialize .dwi")
            }
            _ => panic!("Unsupported extension"),
        }
    } else {
        panic!("Couldn't read extension for simfile");
    };
    file_path.set_extension("sm");
    let mut out_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(file_path)
        .expect("Could not create output file");
    out_file
        .write(notedata.to_sm_string().as_bytes())
        .expect("Could not write to output file");
}
