use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub section_level: u32,
    pub document_keywords: Option<String>,
    pub output_format: OutputFormat,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    #[default]
    Pdf,
    Svg,
    Png,
}