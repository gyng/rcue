use std::time::Duration;

/// Represents a CUE command in a CUE sheet.
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    /// ignore comment
    Rem(String, String),
    /// Media Catalog Number (MCN) of the disc
    Catalog(String),
    /// Path to the file containing the CD-Text meta-data of the disc
    CdTextFile(String),
    /// Special sub-code flags for a track
    Flags(Vec<String>),
    /// The International Standard Recording Code for a track, typically CCOOOYYSSSSS
    ///
    /// C: Country code (uppercase alphanumeric)
    /// O: Owner code (uppercase alphanumeric)
    /// Y: Year (numeric)
    /// S: Serial number (numeric)
    Isrc(String),
    /// Songwriter for a disc or a track
    Songwriter(String),
    /// Performer for a disc or a track
    Performer(String),
    /// Title for a disc or a track
    Title(String),
    /// (file name, file type) of a disc, to be used by tracks
    File(String, String),
    /// A track on a disc
    Track(String, String),
    /// Defines the index of a track on the disc
    Index(String, String),
    /// Length of a track's pregap, (mm:ss:ff) where each frame is 1/75 of a second
    Pregap(String),
    /// Length of a track's postgap (mm:ss:ff) where each frame is 1/75 of a second
    Postgap(String),
    /// Unknown command
    Unknown(String),
    /// Not a command
    None,
}

/// Represents a TRACK in a [`CueFile`](struct.CueFile.html).
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    /// Track number
    pub no: String,
    /// Track format (eg. AUDIO)
    pub format: String,
    /// Title for the track
    pub title: Option<String>,
    /// Performer for the track
    pub performer: Option<String>,
    /// (index, timestamp)
    pub indices: Vec<(String, Duration)>,
    /// Pregap of the track in `Duration`, converted from frames (75 frames = 1s)
    pub pregap: Option<Duration>,
    /// Postgap of the track in `Duration`, converted from frames (75 frames = 1s)
    pub postgap: Option<Duration>,
    /// (key, value)
    pub comments: Vec<(String, String)>,
    /// International Standard Recording Code, typically CCOOOYYSSSSS
    ///
    /// C: Country code (uppercase alphanumeric)
    /// O: Owner code (uppercase alphanumeric)
    /// Y: Year (numeric)
    /// S: Serial number (numeric)
    pub isrc: Option<String>,
    /// Track special sub-code flags (DCP, 4CH, PRE, SCMS)
    pub flags: Vec<String>,
    /// Songwriter for the track
    pub songwriter: Option<String>,
    /// Raw lines from unhandled fields
    pub unknown: Vec<String>,
}

impl Track {
    /// Constructs a new [`Track`](struct.Track.html).
    pub fn new(no: &str, format: &str) -> Self {
        Self {
            songwriter: None,
            no: no.to_string(),
            format: format.to_string(),
            title: None,
            performer: None,
            pregap: None,
            postgap: None,
            indices: Vec::new(),
            comments: Vec::new(),
            unknown: Vec::new(),
            flags: Vec::new(),
            isrc: None,
        }
    }
}

/// Represents a FILE in a [`Cue`](struct.Cue.html).
#[derive(Clone, Debug, PartialEq)]
pub struct CueFile {
    /// Path to file
    pub file: String,
    /// Format (WAVE, MP3, AIFF, BINARY - little endian, MOTOROLA - big endian)
    /// AIFF, WAVE, MP3 are assumed to be 44.1KHz, 16bit and stereo
    pub format: String,
    /// Tracks in this file
    pub tracks: Vec<Track>,
    /// (key, value)
    pub comments: Vec<(String, String)>,
}

impl CueFile {
    /// Constructs a new CueFile.
    pub fn new(file: &str, format: &str) -> Self {
        Self {
            file: file.to_string(),
            tracks: Vec::new(),
            format: format.to_string(),
            comments: Vec::new(),
        }
    }
}

/// Represents a CUE sheet.
#[derive(Clone, Debug, Default)]
pub struct Cue {
    /// Path to the data used for the following TRACK commands
    pub files: Vec<CueFile>,
    /// Title for the entire disc
    pub title: Option<String>,
    /// Performer for the entire disc
    pub performer: Option<String>,
    /// Songwriter for the entire disc
    pub songwriter: Option<String>,
    /// Filename containing the CD-Text metadata of the disc
    pub cd_text_file: Option<String>,
    /// Media Catalog Number (13 decimal digits)
    pub catalog: Option<String>,
    /// (key, value)
    pub comments: Vec<(String, String)>, // are REM fields unique?
    /// Unparsed lines
    pub unknown: Vec<String>,
}

impl Cue {
    /// Constructs a new Cue.
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            songwriter: None,
            cd_text_file: None,
            title: None,
            performer: None,
            catalog: None,
            comments: Vec::new(),
            unknown: Vec::new(),
        }
    }
}
