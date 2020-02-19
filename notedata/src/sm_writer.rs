use crate::{BeatPair, Fraction, Measure, Note, NoteData, NoteRow, NoteType};

pub fn write_sm(data: &NoteData) -> String {
    let mut output = String::new();
    let mut string_tag = |tag_name: &str, from_location: &Option<String>| {
        if let Some(tag) = from_location {
            output.push_str(&write_tag(tag_name, tag))
        }
    };
    string_tag("TITLE", &data.meta.title);
    string_tag("SUBTITLE", &data.meta.subtitle);
    string_tag("ARTIST", &data.meta.artist);
    string_tag("TITLETRANSLIT", &data.meta.title_translit);
    string_tag("SUBTITLETRANSLIT", &data.meta.subtitle_translit);
    string_tag("ARTISTTRANSLIT", &data.meta.artist_translit);
    string_tag("CREDIT", &data.meta.credit);
    string_tag("BANNER", &data.meta.banner_path);
    string_tag("BACKGROUND", &data.meta.background_path);
    string_tag("CDTITLE", &data.meta.cd_title);
    string_tag("MUSIC", &data.meta.music_path);
    let mut number_tag = |tag_name: &str, from_location: &Option<f64>| {
        if let Some(tag) = from_location {
            output.push_str(&write_tag(tag_name, &tag.to_string()))
        }
    };
    number_tag("SAMPLESTART", &data.meta.sample_start);
    number_tag("SAMPLELENGTH", &data.meta.sample_length);
    number_tag("OFFSET", &data.meta.offset.map(|x| -x));
    output.push_str(&write_tag("BPMS", &float_pair_tag(&data.meta.bpms)));
    if let Some(tag) = &data.meta.stops {
        output.push_str(&write_tag("STOPS", &float_pair_tag(tag)));
    }
    for (key, value) in &data.meta.custom {
        if key.1 == "sm" {
            output.push_str(&write_tag(&key.0, value))
        }
    }
    for chart in &data.charts {
        output.push_str(&write_tag("NOTES", &chart_string(&chart)))
    }
    output
}

fn write_tag(tag_name: &str, contents: &str) -> String {
    format!("#{}:{};\n", tag_name, contents)
}

fn beat_to_float<T>(pair: &BeatPair<T>) -> f64 {
    (f64::from(pair.beat) + (f64::from(*pair.sub_beat.numer()) / f64::from(*pair.sub_beat.denom())))
        * 4.0
}

fn float_pair_tag(list: &[BeatPair<f64>]) -> String {
    let mut output = String::new();
    if let Some(first_bpm) = list.get(0) {
        output.push_str(&format!(
            "{}={}",
            beat_to_float(&first_bpm),
            first_bpm.value
        ))
    }
    for bpm in list.iter().skip(1) {
        output.push_str(&format!(",{}={}", beat_to_float(bpm), bpm.value))
    }
    output
}

fn chart_string(chart: &[Measure]) -> String {
    let mut output = String::new();
    output.push_str("dance-single:\n:\nBeginner:\n1:\n0.0,0.0,0.0,0.0,0.0:\n");
    for measure in chart {
        output.push_str(&measure_string(measure, 4));
    }
    output
}

fn measure_string(measure: &[(NoteRow, Fraction)], length: usize) -> String {
    let mut output = String::new();
    let possible_snaps = [4, 8, 12, 16, 24, 32, 48, 64];
    let mut snap_index = 0;
    for (_notes, timestamp) in measure {
        if let Some(snap) = possible_snaps.get(snap_index) {
            if snap % timestamp.denom() != 0 {
                snap_index += 1
            }
        } else {
            break;
        }
    }
    let snap = *possible_snaps.get(snap_index).unwrap_or(&192);
    let mut row_index = 0;
    let blank_line = vec!['0'; length].into_iter().collect::<String>();
    for (notes, timestamp) in measure {
        while Fraction::new(row_index, snap) < *timestamp {
            output.push_str(&format!("{}\n", blank_line));
            row_index += 1;
        }
        output.push_str(&format!("{}\n", row_string(notes, length)));
        row_index += 1;
    }
    while row_index < snap {
        output.push_str(&format!("{}\n", blank_line));
        row_index += 1;
    }
    output.push_str(",\n");
    output
}

fn row_string(row: &[Note], length: usize) -> String {
    let mut blank_line = vec!['0'; length];
    for note in row {
        if let Some(index) = blank_line.get_mut(note.column) {
            *index = notetype_to_char(note.note_type)
        }
    }
    blank_line.into_iter().collect()
}

fn notetype_to_char(note: NoteType) -> char {
    match note {
        NoteType::Tap => '1',
        NoteType::Hold => '2',
        NoteType::HoldEnd => '3',
        NoteType::Roll => '4',
        NoteType::Mine => 'M',
        NoteType::Lift => 'L',
        NoteType::Fake => 'F',
    }
}

#[cfg(test)]
mod tests {
    use crate::sm_writer::write_tag;

    #[test]
    fn create_tag() {
        assert_eq!(write_tag("TITLE", "foobar baz"), "#TITLE:foobar baz;\n")
    }
}
