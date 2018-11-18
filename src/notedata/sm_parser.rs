use super::*;
use nom::double_s;

//This parser should be rewritten, as the current solution is inelegant and likely incurs a
//performance cost as a result.

pub fn parse_tag(tag: &str, contents: &str, data: &mut NoteData) {
    match tag {
        "TITLE" => data.data.title = Some(contents.to_string()),
        "MUSIC" => data.data.music_path = Some(contents.to_string()),
        "OFFSET" => {
            data.data.offset = match contents.parse::<f64>() {
                Ok(thing) => Some(-1.0 * thing),
                Err(_) => None,
            }
        }
        "BPMS" => {
            data.data.bpms = match bpm_parse(&format!("{};", contents)) {
                Ok(thing) => thing
                    .1
                    .into_iter()
                    .map(|(x, y)| {
                        let time_beater = Rational32::approximate_float(x as f64).expect(
                            "Failed to parse bpm time, write real error handling for this later.",
                        );
                        (time_beater.floor().to_integer(), time_beater.fract(), y)
                    }).collect(),
                Err(_) => Vec::new(),
            }
        }
        "NOTES" => data.notes.push(parse_main_block(contents)),
        _ => {}
    }
}

fn parse_main_block(contents: &str) -> ChartData {
    let forbidden: &[_] = &[';', '\n', '\r'];
    ChartData::new(
        contents
            .trim_right_matches(forbidden)
            .lines()
            .filter(|x| *x != "")
            .skip(5)
            .collect::<Vec<_>>()
            .split(|&x| x == ",")
            .map(|measure| parse_measure(measure))
            .collect::<Vec<_>>(),
    )
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
                parse_line(*beat),
            )
        }).collect()
}

fn parse_line(contents: &str) -> NoteRow {
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

named!(pub break_to_tags<&str, Vec<(&str,&str)>>,many0!(complete!(read_sm_tag)));

named!(read_sm_tag<&str,(&str,&str)>,
    do_parse!(
        take_until_and_consume!("#") >>
        name: take_until_and_consume!(":") >>
        contents: take_until!(";") >>
        (name, contents)
    ));

named!( bpm_parse<&str,Vec<(f64,f64)>>, separated_list!(tag!(","), bpm_line));

named!(bpm_line<&str, (f64,f64)>,
  ws!(do_parse!(
        time: double_s >>
           tag!("=")   >>
           bpm: double_s >>
    ( time / 4.0, bpm ) )
  )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lines_correctly() {
        assert_eq!(NoteRow { row: vec![] }, parse_line("0000"));
        assert_eq!(
            NoteRow {
                row: vec![(NoteType::Tap, 2)],
            },
            parse_line("0010")
        );
    }
}
