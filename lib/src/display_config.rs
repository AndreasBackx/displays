#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayConfigs {
    pub displays: Vec<DisplayConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayConfig {
    pub name: String,
    pub path: Option<String>,
    pub is_enabled: bool,
}
