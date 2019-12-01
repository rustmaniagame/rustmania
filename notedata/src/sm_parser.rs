use crate::{
    parser_generic::{beat_pair, comma_separated, stepmania_tag, ws_trimmed},
    Chart, DisplayBpm, Measure, Note, NoteData, NoteRow, NoteType,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace1, none_of, not_line_ending},
    combinator::map,
    error::ErrorKind,
    multi::{count, fold_many0, fold_many1, many0, separated_nonempty_list},
    number::complete::double,
    sequence::{preceded, separated_pair, terminated},
    Err, IResult,
};
use num_rational::Rational32;

fn offset(input: &str) -> IResult<&str, f64> {
    map(double, |value| -value)(input)
}

fn display_bpm(input: &str) -> IResult<&str, DisplayBpm> {
    alt((
        map(
            separated_pair(double, ws_trimmed(char(':')), double),
            |(min, max)| DisplayBpm::Range(min, max),
        ),
        map(double, DisplayBpm::Static),
        map(char('*'), |_| DisplayBpm::Random),
    ))(input)
}

fn notetype(input: &str) -> IResult<&str, Option<NoteType>> {
    map(none_of("\r\n,"), into_sm_notetype)(input)
}

fn into_sm_notetype(sm_char: char) -> Option<NoteType> {
    match sm_char {
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

fn noterow(input: &str) -> IResult<&str, NoteRow> {
    map(
        fold_many1(notetype, (vec![], 0), |(mut noterow, mut index), item| {
            if let Some(item) = item {
                noterow.push(Note::new(item, index))
            }
            index += 1;
            (noterow, index)
        }),
        |(noterow, _)| noterow,
    )(input)
}

fn measure(input: &str) -> IResult<&str, Measure> {
    map(
        fold_many0(
            terminated(noterow, multispace1),
            (vec![], 0),
            |(mut noterows, mut index), item| {
                if !item.is_empty() {
                    noterows.push((item, index))
                }
                index += 1;
                (noterows, index)
            },
        ),
        |(noterows, total)| {
            noterows
                .into_iter()
                .map(|(item, index)| (item, Rational32::new(index, total)))
                .collect()
        },
    )(input)
}

fn chart(input: &str) -> IResult<&str, Chart> {
    preceded(
        terminated(
            count(terminated(take_until(":"), char(':')), 5),
            many0(alt((comment, multispace1))),
        ),
        separated_nonempty_list(
            preceded(
                many0(alt((comment, multispace1))),
                terminated(char(','), many0(alt((comment, multispace1)))),
            ),
            measure,
        ),
    )(input)
}

fn comment(input: &str) -> IResult<&str, &str> {
    preceded(tag("//"), not_line_ending)(input)
}

fn notedata(input: &str) -> IResult<&str, NoteData> {
    let mut input = input;
    let mut nd = NoteData::new();

    while let Ok((output, (tag, value))) = preceded(take_until("#"), stepmania_tag)(input) {
        input = output;

        if !value.trim().is_empty() {
            match tag {
                "TITLE" => nd.meta.title = Some(value.to_owned()),
                "SUBTITLE" => nd.meta.subtitle = Some(value.to_owned()),
                "ARTIST" => nd.meta.artist = Some(value.to_owned()),
                "TITLETRANSLIT" => nd.meta.title_translit = Some(value.to_owned()),
                "SUBTITLETRANSLIT" => nd.meta.subtitle_translit = Some(value.to_owned()),
                "ARTISTTRANSLIT" => nd.meta.artist_translit = Some(value.to_owned()),
                "GENRE" => nd.meta.genre = Some(value.to_owned()),
                "CREDIT" => nd.meta.credit = Some(value.to_owned()),
                "BANNER" => nd.meta.banner_path = Some(value.to_owned()),
                "BACKGROUND" => nd.meta.background_path = Some(value.to_owned()),
                "LYRICSPATH" => nd.meta.lyrics_path = Some(value.to_owned()),
                "CDTITLE" => nd.meta.cd_title = Some(value.to_owned()),
                "MUSIC" => nd.meta.music_path = Some(value.to_owned()),
                "SAMPLESTART" => nd.meta.sample_start = Some(ws_trimmed(double)(value)?.1),
                "SAMPLELENGTH" => nd.meta.sample_length = Some(ws_trimmed(double)(value)?.1),
                "OFFSET" => nd.meta.offset = Some(ws_trimmed(offset)(value)?.1),
                "DISPLAYBPM" => nd.meta.display_bpm = Some(ws_trimmed(display_bpm)(value)?.1),
                "BPMS" => nd.meta.bpms = ws_trimmed(comma_separated(beat_pair(double)))(value)?.1,
                "STOPS" => {
                    nd.meta.stops = Some(ws_trimmed(comma_separated(beat_pair(double)))(value)?.1)
                }
                "NOTES" => nd.charts.push(chart(value)?.1),
                _ => {}
            }
        }
    }
    Ok((input, nd))
}

pub fn parse(input: &str) -> Result<NoteData, Err<(&str, ErrorKind)>> {
    notedata(input).map(|notedata| notedata.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BeatPair, ChartMetadata};
    use nom::Err::Error;

    #[test]
    fn parse_offset() {
        assert_eq!(offset("1.2  foo"), Ok(("  foo", -1.2)));
        assert_eq!(offset("-3.4foo"), Ok(("foo", 3.4)));
    }

    #[test]
    fn parse_display_bpm() {
        assert_eq!(
            display_bpm("1.2  :   3.4  foo"),
            Ok(("  foo", DisplayBpm::Range(1.2, 3.4)))
        );
        assert_eq!(display_bpm("1.2foo"), Ok(("foo", DisplayBpm::Static(1.2))));
        assert_eq!(display_bpm("*"), Ok(("", DisplayBpm::Random)));
    }

    #[test]
    fn parse_notetype() {
        assert_eq!(notetype("1foo"), Ok(("foo", Some(NoteType::Tap))));
        assert_eq!(notetype("2foo"), Ok(("foo", Some(NoteType::Hold))));
        assert_eq!(notetype("3foo"), Ok(("foo", Some(NoteType::HoldEnd))));
        assert_eq!(notetype("4foo"), Ok(("foo", Some(NoteType::Roll))));
        assert_eq!(notetype("Mfoo"), Ok(("foo", Some(NoteType::Mine))));
        assert_eq!(notetype("Lfoo"), Ok(("foo", Some(NoteType::Lift))));
        assert_eq!(notetype("Ffoo"), Ok(("foo", Some(NoteType::Fake))));
        assert_eq!(notetype("0foo"), Ok(("foo", None)));
        assert_eq!(notetype("\rfoo"), Err(Error(("\rfoo", ErrorKind::NoneOf))));
        assert_eq!(notetype("\nfoo"), Err(Error(("\nfoo", ErrorKind::NoneOf))));
        assert_eq!(notetype(",foo"), Err(Error((",foo", ErrorKind::NoneOf))));
    }

    #[test]
    fn parse_noterow() {
        assert_eq!(
            noterow("0101\n"),
            Ok((
                "\n",
                vec![Note::new(NoteType::Tap, 1), Note::new(NoteType::Tap, 3)]
            ))
        );
    }

    #[test]
    fn parse_measure() {
        assert_eq!(
            measure(
                "0000\n \
                 0100\n \
                 0000\n \
                 0010\n \
                 0000\n"
            ),
            Ok((
                "",
                vec![
                    (vec![Note::new(NoteType::Tap, 1)], Rational32::new(1, 5)),
                    (vec![Note::new(NoteType::Tap, 2)], Rational32::new(3, 5)),
                ]
            ))
        );
    }

    #[test]
    fn parse_comment() {
        assert_eq!(comment("// foo\nbar"), Ok(("\nbar", " foo")));
    }

    #[test]
    fn parse_chart() {
        assert_eq!(
            chart(
                "
                      foo::
                      bar ::
                      0.000,0.000  :\n\n \
                 0000\n \
                 0100\n \
                 0000\n \
                 0000\n \
                 , // baz\n
                 0000\n \
                 0000\n \
                 0010\n \
                 0000\n"
            ),
            Ok((
                "",
                vec![
                    vec![(vec![Note::new(NoteType::Tap, 1)], Rational32::new(1, 4))],
                    vec![(vec![Note::new(NoteType::Tap, 2)], Rational32::new(1, 2))],
                ]
            ))
        );
    }

    #[test]
    fn parse_notedata() {
        assert_eq!(
            notedata(
                "content that is

        #TITLE:bar1;

        not part of a tag is discarded

        #SUBTITLE:bar2;#ARTIST:bar3;#TITLETRANSLIT:bar4;#SUBTITLETRANSLIT:bar5;
        #ARTISTTRANSLIT:bar6;#GENRE:bar7;#CREDIT:bar8;#BANNER:bar9;
        #BACKGROUND:bar10;#LYRICSPATH:bar11;#CDTITLE:bar12;#MUSIC:bar13;
        #SAMPLESTART:  1.2 ;#SAMPLELENGTH: 3.4  ;#BPMS:  1.0=2 ;
        #STOPS: 3.0=4  ;#OFFSET:  1 ;#DISPLAYBPM: *  ;#STOPS:
        ;
        #NOTES: ::::: \
            0000\n \
            0100\n \
            0000\n \
            0000\n \
            ;
        #NOTES: ::::: \
            0000\n \
            0000\n \
            0010\n \
            0000\n \
            ;"
            ),
            Ok((
                "",
                NoteData {
                    meta: ChartMetadata {
                        title: Some("bar1".to_owned()),
                        subtitle: Some("bar2".to_owned()),
                        artist: Some("bar3".to_owned()),
                        title_translit: Some("bar4".to_owned()),
                        subtitle_translit: Some("bar5".to_owned()),
                        artist_translit: Some("bar6".to_owned()),
                        genre: Some("bar7".to_owned()),
                        credit: Some("bar8".to_owned()),
                        banner_path: Some("bar9".to_owned()),
                        background_path: Some("bar10".to_owned()),
                        lyrics_path: Some("bar11".to_owned()),
                        cd_title: Some("bar12".to_owned()),
                        music_path: Some("bar13".to_owned()),
                        sample_start: Some(1.2),
                        sample_length: Some(3.4),
                        bpms: vec![BeatPair::from_pair(1., 2.).unwrap()],
                        stops: Some(vec![BeatPair::from_pair(3., 4.).unwrap()]),
                        offset: Some(-1.),
                        display_bpm: Some(DisplayBpm::Random),
                        background_changes: None,
                        foreground_changes: None,
                        selectable: None,
                    },
                    charts: vec![
                        vec![vec![(
                            vec![Note::new(NoteType::Tap, 1)],
                            Rational32::new(1, 4),
                        )]],
                        vec![vec![(
                            vec![Note::new(NoteType::Tap, 2)],
                            Rational32::new(2, 4),
                        )]],
                    ],
                }
            ))
        );
    }
}
