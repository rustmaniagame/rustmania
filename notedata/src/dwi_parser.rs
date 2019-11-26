use crate::{
    parser_generic::{beat_pair, comma_separated, stepmania_tag, ws_trimmed},
    BeatPair, NoteData,
};
use nom::{
    bytes::complete::take_until, error::ErrorKind, number::complete::double, sequence::preceded,
    Err, IResult,
};

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
                "BPM" => {
                    let beat_pair = BeatPair::from_pair(0.0, ws_trimmed(double)(value)?.1)
                        .expect("Could not parse initial bpm into internal format");
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
        #BPM:123.4;
        #CHANGEBPM:23.4=56.7,256=128;"
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
                        genre: None,
                        credit: None,
                        banner_path: None,
                        background_path: None,
                        lyrics_path: None,
                        cd_title: None,
                        music_path: None,
                        sample_start: None,
                        sample_length: None,
                        bpms: vec![
                            BeatPair::from_pair(0.0, 123.4).unwrap(),
                            BeatPair::from_pair(23.4, 56.7).unwrap(),
                            BeatPair::from_pair(256.0, 128.0).unwrap()
                        ],
                        stops: None,
                        offset: None,
                        display_bpm: None,
                        background_changes: None,
                        foreground_changes: None,
                        selectable: None,
                    },
                    charts: vec![],
                }
            ))
        );
    }
}
