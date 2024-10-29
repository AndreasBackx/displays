#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayConfigs {
    pub displays: Vec<DisplayConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayConfig {
    pub name: String,
    pub path: Option<String>,
    pub enabled: bool,
}
