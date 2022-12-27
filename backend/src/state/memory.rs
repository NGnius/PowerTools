#[derive(Debug, Clone)]
pub struct Memory {
    pub transparent_hugepages_set: bool,
}

impl std::default::Default for Memory {
    fn default() -> Self {
        Self {
            transparent_hugepages_set: false,
        }
    }
}
