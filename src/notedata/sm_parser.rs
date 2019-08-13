use super::*;
use nom::{
    complete, do_parse, double, many0, named, separated_list, tag, take_until,
    take_until_and_consume, ws,
};

//Needs: DISPLAYBPM, SELECTABLE, BGCHANGES, FGCHANGES
pub fn parse_tag(tag: &str, contents: &str, data: &mut NoteData) {
    match tag {
        "TITLE" => data.data.title = Some(contents.to_string()),
        "SUBTITLE" => data.data.subtitle = Some(contents.to_string()),
        "ARTIST" => data.data.artist = Some(contents.to_string()),
        "TITLETRANSLIT" => data.data.title_translit = Some(contents.to_string()),
        "SUBTITLETRANSLIT" => data.data.subtitle_translit = Some(contents.to_string()),
        "ARTISTTRANSLIT" => data.data.artist_translit = Some(contents.to_string()),
        "GENRE" => data.data.genre = Some(contents.to_string()),
        "CREDIT" => data.data.credit = Some(contents.to_string()),
        "BANNER" => data.data.banner_path = Some(contents.to_string()),
        "BACKGROUND" => data.data.background_path = Some(contents.to_string()),
        "LYRICSPATH" => data.data.lyrics_path = Some(contents.to_string()),
        "CDTITLE" => data.data.cd_title = Some(contents.to_string()),
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
                        let time_beater = Rational32::approximate_float(x as f64)
                            .expect("Failed to parse bpm time.");
                        (time_beater.floor().to_integer(), time_beater.fract(), y)
                    })
                    .collect(),
                Err(_) => Vec::new(),
            }
        }
        "STOPS" => {
            data.data.stops = match bpm_parse(&format!("{};", contents)) {
                Ok(thing) => Some(
                    thing
                        .1
                        .into_iter()
                        .map(|(x, y)| {
                            let time_beater = Rational32::approximate_float(x as f64)
                                .expect("Failed to parse stop time.");
                            (time_beater.floor().to_integer(), time_beater.fract(), y)
                        })
                        .collect(),
                ),
                Err(_) => None,
            }
        }
        "SAMPLESTART" => data.data.sample_start = contents.parse().ok(),
        "SAMPLELENGTH" => data.data.sample_length = contents.parse().ok(),
        "NOTES" => data.notes.push(parse_main_block(contents)),
        _ => {}
    }
}

fn parse_main_block(contents: &str) -> ChartData {
    let forbidden: &[_] = &[';', '\n', '\r'];
    ChartData::new(
        contents
            .trim_end_matches(forbidden)
            .lines()
            .filter(|x| *x != "")
            .skip(5)
            .collect::<Vec<_>>()
            .split(|&x| x.starts_with(','))
            .map(|measure| parse_measure(measure))
            .collect::<Vec<_>>(),
    )
}

fn char_to_notetype(character: char) -> Option<NoteType> {
    match character {
        '1' => Some(NoteType::Tap),
        '2' => Some(NoteType::Hold),
        '3' => Some(NoteType::HoldEnd),
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
        })
        .filter(|(_, x)| !x.row.is_empty())
        .collect()
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
           time: double >>
               tag!("=")   >>
               bpm: double >>
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
