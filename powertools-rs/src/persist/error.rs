#[derive(Debug)]
pub enum JsonError {
    Serde(serde_json::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Serde(e) => (e as &dyn std::fmt::Display).fmt(f),
            Self::Io(e) => (e as &dyn std::fmt::Display).fmt(f),
        }
    }
}
