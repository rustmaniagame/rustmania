pub struct NoteLayout {
    pub column_positions: [u32; 4],
}

impl NoteLayout {
    pub fn new() -> NoteLayout {
        NoteLayout {
            column_positions: [300, 350, 400, 450],
        }
    }
}