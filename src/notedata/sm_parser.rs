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
        "NOTES" => data.notes = parse_main_block(contents),
        _ => {}
    }
}

fn parse_main_block(contents: &str) -> Vec<Vec<(Rational32, NoteRow)>> {
    contents
        .lines()
        .skip(6)
        .collect::<Vec<_>>()
        .split(|&x| x == ",")
        .map(|measure| parse_measure(measure))
        .collect::<Vec<_>>()
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

fn parse_measure(measure: &[&str]) -> Vec<(Rational32, NoteRow)> {
    let division = measure.len();
    measure
        .iter()
        .enumerate()
        .map(|(subindex, beat)| {
            (
                Rational32::new(subindex as i32, division as i32),
                parse_line(beat),
            )
        })
        .collect()
}

fn parse_line(contents: &&str) -> NoteRow {
    NoteRow {
        row: contents
            .chars()
            .enumerate()
            .map(|(index, character)| (char_to_notetype(character), index))
            .filter(|(index, _)| index.is_some())
            .map(|(index, character)| (index.unwrap(), character))
            .collect(),
    }
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
