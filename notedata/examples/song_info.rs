use notedata::{Chart, NoteType};
use std::env::current_dir;
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

fn count_of_chord_size(chart: &Chart, size: usize) -> usize {
    chart
        .iter()
        .flatten()
        .filter(|(notes, _)| {
            notes
                .iter()
                .filter(|note| note.note_type == NoteType::Tap || note.note_type == NoteType::Hold)
                .count()
                == size
        })
        .count()
}

fn main() {
    let mut sim = String::new();
    if let Ok(path) = current_dir() {
        println!("Paths are relative to {:?}.\n", path);
    } else {
        println!("Could not determine program location.\n")
    }
    print!("Enter simfile path: ");
    let _ = io::stdout().flush();
    io::stdin()
        .read_line(&mut sim)
        .expect("failed to read input");
    let sim = PathBuf::from(sim.trim_end_matches('\n'));
    let notedata = if let Some(extension) = sim.extension() {
        let sim = File::open(sim.as_path()).expect("");
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

    if let Some(title) = &notedata.meta.title {
        println!("Song title is: {}", title);
    } else {
        println!("Song has no title");
    }
    if let Some(title) = &notedata.meta.artist {
        println!("Song artist is: {}", title);
    } else {
        println!("Song has no artist info");
    }
    if !notedata.meta.bpms.is_empty() {
        print!("Bpms are: {}", notedata.meta.bpms[0].value);
        for bpm in notedata.meta.bpms.iter().skip(1) {
            print!(", {}", bpm.value);
        }
        println!();
    } else {
        println!("Song has no BPM info");
    }
    if let Some(offset) = notedata.meta.offset {
        println!("Song offset is: {}", offset);
    } else {
        println!("Song has no offset");
    }

    for chart in &notedata.charts {
        println!();
        println!(
            "Total number of notes: {}",
            chart
                .iter()
                .flatten()
                .map(|(notes, _)| notes
                    .iter()
                    .filter(
                        |note| note.note_type == NoteType::Tap || note.note_type == NoteType::Hold
                    )
                    .count())
                .sum::<usize>()
        );
        println!("Total taps: {}", count_of_chord_size(chart, 1));
        println!("Total jumps: {}", count_of_chord_size(chart, 2));
        println!("Total hands: {}", count_of_chord_size(chart, 3));
        println!("Total quads: {}", count_of_chord_size(chart, 4));
    }
}
