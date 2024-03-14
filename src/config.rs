use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_section_level")]
    pub section_level: u32,
    pub document_keywords: Option<String>,
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_output_filename")]
    pub output_filename: String,
    #[serde(default = "default_template_path")]
    pub template_path: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    #[default]
    Pdf,
    Svg,
    Png,
}

fn default_section_level() -> u32 { 3 }

fn default_output_format() -> OutputFormat { OutputFormat::Pdf }

fn default_output_dir() -> String { "typst-piggsoft".to_string() }

fn default_output_filename() -> String { "out".to_string() }

fn default_template_path() -> Option<String> { None }
