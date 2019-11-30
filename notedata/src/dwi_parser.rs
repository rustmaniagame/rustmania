use crate::{
    parser_generic::{beat_pair, comma_separated, stepmania_tag, ws_trimmed},
    BeatPair, DisplayBpm, Fraction, Measure, Note, NoteData, NoteRow, NoteType,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{char, multispace0},
    combinator::map,
    error::ErrorKind,
    multi::{count, fold_many0, fold_many_m_n},
    number::complete::double,
    sequence::{preceded, separated_pair, terminated},
    Err, IResult,
};

fn display_bpm_dwi(input: &str) -> IResult<&str, DisplayBpm> {
    Ok(alt((
        map(
            //we accept either .. or . as a separator because you can have the first bpm as an integer
            //and the first . gets read into the first parser as a result
            separated_pair(double, ws_trimmed(alt((tag(".."), tag(".")))), double),
            |(min, max)| DisplayBpm::Range(min, max),
        ),
        map(double, DisplayBpm::Static),
    ))(input)
    .unwrap_or(("", DisplayBpm::Random)))
}

fn dwi_noterow(input: &str) -> IResult<&str, NoteRow> {
    match input.chars().next() {
        //5 should not appear in normal dwi files, but it can be parsed by stepmania 5
        Some('0') | Some('5') => Ok((&input[1..], vec![])),
        Some('4') => Ok((
            &input[1..],
            vec![Note {
                note_type: NoteType::Tap,
                column: 0,
            }],
        )),
        Some('2') => Ok((
            &input[1..],
            vec![Note {
                note_type: NoteType::Tap,
                column: 1,
            }],
        )),
        Some('8') => Ok((
            &input[1..],
            vec![Note {
                note_type: NoteType::Tap,
                column: 2,
            }],
        )),
        Some('6') => Ok((
            &input[1..],
            vec![Note {
                note_type: NoteType::Tap,
                column: 3,
            }],
        )),
        Some('1') => Ok((
            &input[1..],
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 0,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 1,
                },
            ],
        )),
        Some('7') => Ok((
            &input[1..],
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 0,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 2,
                },
            ],
        )),
        Some('B') => Ok((
            &input[1..],
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 0,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 3,
                },
            ],
        )),
        Some('A') => Ok((
            &input[1..],
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 1,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 2,
                },
            ],
        )),
        Some('3') => Ok((
            &input[1..],
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 1,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 3,
                },
            ],
        )),
        Some('9') => Ok((
            &input[1..],
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 2,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 3,
                },
            ],
        )),
        _ => Err(Err::Error((input, ErrorKind::Char))),
    }
}

fn dwi_measure_n(input: &str, n: usize) -> IResult<&str, Measure> {
    fold_many_m_n(
        n,
        n,
        alt((dwi_noterow, dwi_chord)),
        (vec![], 0),
        |(mut acc, idx), item| {
            if !item.is_empty() {
                acc.push((item, Fraction::new(idx, n as i32)));
            }
            (acc, idx + 1)
        },
    )(input)
    .map(|(x, (y, _))| (x, y))
}

fn dwi_measure_8(input: &str) -> IResult<&str, Measure> {
    dwi_measure_n(input, 8)
}

fn dwi_measure_16(input: &str) -> IResult<&str, Measure> {
    dwi_measure_n(input, 16)
}

fn dwi_measure_24(input: &str) -> IResult<&str, Measure> {
    dwi_measure_n(input, 24)
}

fn dwi_measure_64(input: &str) -> IResult<&str, Measure> {
    dwi_measure_n(input, 64)
}

fn dwi_measure_192(input: &str) -> IResult<&str, Measure> {
    dwi_measure_n(input, 192)
}

fn dwi_measure(input: &str) -> IResult<&str, Measure> {
    match input.chars().next() {
        Some('(') => terminated(dwi_measure_16, char(')'))(&input[1..]),
        Some('[') => terminated(dwi_measure_24, char(']'))(&input[1..]),
        Some('{') => terminated(dwi_measure_64, char('}'))(&input[1..]),
        Some('`') => terminated(dwi_measure_192, char('\''))(&input[1..]),
        Some(_) => dwi_measure_8(input),
        None => Err(Err::Error((input, ErrorKind::Eof))),
    }
}

fn dwi_chart(input: &str) -> IResult<&str, Vec<Measure>> {
    fold_many0(
        preceded(multispace0, dwi_measure),
        vec![],
        |mut acc, measure| {
            acc.push(measure);
            acc
        },
    )(input)
}

fn dwi_chord(input: &str) -> IResult<&str, NoteRow> {
    terminated(
        preceded(
            char('<'),
            fold_many0(dwi_noterow, vec![], |mut acc, item| {
                if !item.is_empty() {
                    acc.push(item);
                }
                acc
            }),
        ),
        char('>'),
    )(input)
    .map(|(input, output)| {
        let mut collected: Vec<Note> = output.into_iter().flatten().collect();
        collected.sort_by(|x, y| x.column.cmp(&y.column));
        collected.dedup_by(|x, y| x.column == y.column);
        (input, collected)
    })
}

pub fn parse(input: &str) -> Result<NoteData, Err<(&str, ErrorKind)>> {
    notedata(input).map(|notedata| notedata.1)
}

fn notedata(input: &str) -> IResult<&str, NoteData> {
    let mut input = input;
    let mut nd = NoteData::new();

    while let Ok((output, (tag, value))) = preceded(take_until("#"), stepmania_tag)(input) {
        input = output;

        if !value.trim().is_empty() {
            match tag {
                "TITLE" => nd.meta.title = Some(value.to_owned()),
                "ARTIST" => nd.meta.artist = Some(value.to_owned()),
                "GENRE" => nd.meta.genre = Some(value.to_owned()),
                "CDTITLE" => nd.meta.cd_title = Some(value.to_owned()),
                "FILE" => nd.meta.music_path = Some(value.to_owned()),
                "BPM" => {
                    let beat_pair = BeatPair::at_start(ws_trimmed(double)(value)?.1);
                    if let Some(bpm) = nd.meta.bpms.get_mut(0) {
                        *bpm = beat_pair
                    } else {
                        nd.meta.bpms = vec![beat_pair];
                    }
                }
                "CHANGEBPM" => {
                    if nd.meta.bpms.is_empty() {
                        nd.meta.bpms.push(BeatPair::from_pair(0.0, 120.0).unwrap())
                    }
                    nd.meta
                        .bpms
                        .append(&mut ws_trimmed(comma_separated(beat_pair(double)))(value)?.1)
                }
                "DISPLAYBPM" => nd.meta.display_bpm = Some(ws_trimmed(display_bpm_dwi)(value)?.1),
                "SINGLE" => nd.charts.push(
                    preceded(
                        terminated(
                            count(terminated(take_until(":"), char(':')), 2),
                            multispace0,
                        ),
                        dwi_chart,
                    )(value)?
                    .1,
                ),
                _ => {}
            }
        }
    }
    Ok((input, nd))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChartMetadata;

    #[test]
    fn parse_notedata() {
        assert_eq!(
            notedata(
                "content that is

        #TITLE:bar1;

        not part of a tag is discarded

        #SUBTITLE:bar2;#ARTIST:bar3;
        #GENRE:bar4;
        #CDTITLE:bar5;
        #FILE:bar6.mp3;
        #BPM:123.4;
        #DISPLAYBPM:100..200;
        #CHANGEBPM:23.4=56.7,256=128;
        #SINGLE:SMANIC:17:
        00004008
        (<42>000100060000000);"
            ),
            Ok((
                "",
                NoteData {
                    meta: ChartMetadata {
                        title: Some("bar1".to_owned()),
                        subtitle: None,
                        artist: Some("bar3".to_owned()),
                        title_translit: None,
                        subtitle_translit: None,
                        artist_translit: None,
                        genre: Some("bar4".to_owned()),
                        credit: None,
                        banner_path: None,
                        background_path: None,
                        lyrics_path: None,
                        cd_title: Some("bar5".to_owned()),
                        music_path: Some("bar6.mp3".to_owned()),
                        sample_start: None,
                        sample_length: None,
                        bpms: vec![
                            BeatPair::from_pair(0.0, 123.4).unwrap(),
                            BeatPair::from_pair(23.4, 56.7).unwrap(),
                            BeatPair::from_pair(256.0, 128.0).unwrap()
                        ],
                        stops: None,
                        offset: None,
                        display_bpm: Some(DisplayBpm::Range(100., 200.)),
                        background_changes: None,
                        foreground_changes: None,
                        selectable: None,
                    },
                    charts: vec![vec![
                        vec![
                            (
                                vec![Note {
                                    note_type: NoteType::Tap,
                                    column: 0
                                }],
                                Fraction::new(1, 2)
                            ),
                            (
                                vec![Note {
                                    note_type: NoteType::Tap,
                                    column: 2
                                }],
                                Fraction::new(7, 8)
                            )
                        ],
                        vec![
                            (
                                vec![
                                    Note {
                                        note_type: NoteType::Tap,
                                        column: 0
                                    },
                                    Note {
                                        note_type: NoteType::Tap,
                                        column: 1
                                    }
                                ],
                                Fraction::new(0, 1)
                            ),
                            (
                                vec![
                                    Note {
                                        note_type: NoteType::Tap,
                                        column: 0
                                    },
                                    Note {
                                        note_type: NoteType::Tap,
                                        column: 1
                                    }
                                ],
                                Fraction::new(1, 4)
                            ),
                            (
                                vec![Note {
                                    note_type: NoteType::Tap,
                                    column: 3
                                }],
                                Fraction::new(1, 2)
                            )
                        ]
                    ]],
                }
            ))
        );
    }
    #[test]
    fn parse_measure() {
        assert_eq!(
            Ok((
                "\n",
                vec![
                    (
                        vec![
                            Note {
                                note_type: NoteType::Tap,
                                column: 0
                            },
                            Note {
                                note_type: NoteType::Tap,
                                column: 1
                            }
                        ],
                        Fraction::new(0, 1)
                    ),
                    (
                        vec![
                            Note {
                                note_type: NoteType::Tap,
                                column: 0
                            },
                            Note {
                                note_type: NoteType::Tap,
                                column: 2
                            },
                            Note {
                                note_type: NoteType::Tap,
                                column: 3
                            }
                        ],
                        Fraction::new(3, 8)
                    ),
                    (
                        vec![Note {
                            note_type: NoteType::Tap,
                            column: 2
                        }],
                        Fraction::new(3, 4)
                    ),
                ]
            )),
            dwi_measure("100<49>5080\n")
        );
        assert_eq!(
            Ok((
                "\n",
                vec![
                    (
                        vec![
                            Note {
                                note_type: NoteType::Tap,
                                column: 0
                            },
                            Note {
                                note_type: NoteType::Tap,
                                column: 2
                            },
                            Note {
                                note_type: NoteType::Tap,
                                column: 3
                            }
                        ],
                        Fraction::new(7, 16)
                    ),
                    (
                        vec![
                            Note {
                                note_type: NoteType::Tap,
                                column: 0
                            },
                            Note {
                                note_type: NoteType::Tap,
                                column: 3
                            }
                        ],
                        Fraction::new(7, 8)
                    ),
                ]
            )),
            dwi_measure("(0000000<94>005000B0)\n")
        );
        assert_eq!(
            Ok((
                "\n",
                vec![(
                    vec![Note {
                        note_type: NoteType::Tap,
                        column: 1
                    },],
                    Fraction::new(1, 24)
                )]
            )),
            dwi_measure("[020000000000000000000000]\n")
        );
        assert_eq!(
            Ok((
                "\n",
                vec![(
                    vec![Note {
                        note_type: NoteType::Tap,
                        column: 1
                    },],
                    Fraction::new(1, 64)
                )]
            )),
            dwi_measure("{0200000000000000000000000000000000000000000000000000000000000000}\n")
        );
        assert_eq!(
            Ok((
                "\n",
                vec![(
                    vec![Note {
                        note_type: NoteType::Tap,
                        column: 1
                    },],
                    Fraction::new(1, 192)
                )]
            )),
            dwi_measure("`02000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000\'\n")
        );
    }

    #[test]
    fn parse_chord() {
        assert_eq!(Ok(("", vec![])), dwi_chord("<>"));
        let hand_example = Ok((
            "",
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 0,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 1,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 3,
                },
            ],
        ));
        assert_eq!(hand_example, dwi_chord("<16>"));
        assert_eq!(hand_example, dwi_chord("<61>"));
        assert_eq!(hand_example, dwi_chord("<34>"));
        assert_eq!(hand_example, dwi_chord("<B2>"));
        assert_eq!(hand_example, dwi_chord("<31>"));
        assert_eq!(hand_example, dwi_chord("<426>"));
        let quad_example = Ok((
            "",
            vec![
                Note {
                    note_type: NoteType::Tap,
                    column: 0,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 1,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 2,
                },
                Note {
                    note_type: NoteType::Tap,
                    column: 3,
                },
            ],
        ));
        assert_eq!(quad_example, dwi_chord("<BA>"));
        assert_eq!(quad_example, dwi_chord("<AB>"));
        assert_eq!(quad_example, dwi_chord("<91>"));
        assert_eq!(quad_example, dwi_chord("<816>"));
        assert_eq!(quad_example, dwi_chord("<6428>"));
        assert_eq!(quad_example, dwi_chord("<97A>"));
        assert_eq!(quad_example, dwi_chord("<B50A>"));
        assert!(dwi_chord("246").is_err());
    }
}
