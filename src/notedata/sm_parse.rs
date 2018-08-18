use super::*;
use nom::double_s;

pub fn parse_tag(tag: &str, contents: &str, data: &mut NoteData) {
    match tag {
        "TITLE" => data.data.title = Some(contents.to_string()),
        "OFFSET" => {
            data.data.offset = match float_tag_parse(contents) {
                Ok(thing) => Some(-1.0 * thing.1),
                Err(_) => None,
            }
        }
        "BPMS" => {
            data.data.bpm = match bpm_parse(contents) {
                Ok(thing) => Some(((thing.1).1).1),
                Err(_) => None,
            }
        }
        "NOTES" => data.notes = parse_main_block(contents.to_string()),
        _ => {}
    }
}

fn parse_main_block(contents: String) -> Vec<Vec<(Fraction, NoteRow)>> {
    let mut notes = Vec::new();
    let lines: Vec<_> = contents.lines().skip(6).collect();
    let measures = lines.split(|&x| x == ",");
    for measure in measures {
        notes.push(parse_measure(measure));
    }
    notes
}

fn char_to_notetype(character: char) -> Option<NoteType> {
    match character {
        '0' => None,
        '1' => Some(NoteType::Tap),
        '2' => Some(NoteType::Hold),
        '4' => Some(NoteType::Roll),
        'M' => Some(NoteType::Mine),
        'L' => Some(NoteType::Lift),
        'F' => Some(NoteType::Fake),
        _ => None,
    }
}

fn parse_measure(measure: &[&str]) -> Vec<(Fraction, NoteRow)> {
    let mut output = Vec::new();
    let division = measure.len();
    for (subindex, beat) in measure.iter().enumerate() {
        output.push((
            Fraction::new(subindex as i64, division as u64).unwrap(),
            parse_line(beat),
        ));
    }
    output
}

fn parse_line(contents: &&str) -> NoteRow {
    let mut row = Vec::new();
    contents.chars().enumerate().for_each(|(index, character)| {
        if let Some(note) = char_to_notetype(character) {
            row.push((note, index));
        }
    });
    NoteRow { row }
}

named!( bpm_parse<&str,(Vec<(f64,f64)>,(f64,f64))>, many_till!(bpm_line, do_parse!(
time: double_s >>
tag!("=")   >>
bpm: double_s >>
tag!(";")    >>
( ( time, bpm ) )
)));

named!(bpm_line<&str, (f64,f64)>,
  do_parse!(
        time: double_s >>
           tag!("=")   >>
           bpm: double_s >>
           tag!(",")    >>
    ( ( time, bpm ) )
  )
);

named!(float_tag_parse<&str, f64>,
    do_parse!(
        value: double_s >>
        tag!(";") >>
    ( value )
));
