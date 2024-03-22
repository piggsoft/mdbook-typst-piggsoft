use std::io;
use std::io::Write;
use std::path::PathBuf;
use crate::config::{Config, OutputFormat};


fn concatenate_name_and_suffix(name: &str, suffix: &str) -> String {
    format!("{}{}", name, suffix)
}

pub fn export(
    config: &Config,
    root_path: &PathBuf,
    typst_file: &PathBuf,
    out_file: &PathBuf,
) {
    let command = match config.output_format {
        OutputFormat::Pdf => {
            let mut c = std::process::Command::new("typst");
            c/*.arg("-v")*/
                .arg("compile")
                .arg("--format")
                .arg("pdf")
                .arg("--root")
                .arg(&root_path)
                .arg(&typst_file)
                .arg(&out_file.join(concatenate_name_and_suffix(&config.output_filename, ".pdf")));
            Some(c)
        }
        OutputFormat::Svg => {
            let mut c = std::process::Command::new("typst");
            c/*.arg("-v")*/
                .arg("compile")
                .arg("--format")
                .arg("svg")
                .arg("--root")
                .arg(&root_path)
                .arg(&typst_file)
                .arg(&out_file.join(concatenate_name_and_suffix(&config.output_filename, "-{n}.svg")));
            Some(c)
        }
        OutputFormat::Png => {
            let mut c = std::process::Command::new("typst");
            c/*.arg("-v")*/
                .arg("compile")
                .arg("--format")
                .arg("png")
                .arg("--root")
                .arg(&root_path)
                .arg(&typst_file)
                .arg(&out_file.join(concatenate_name_and_suffix(&config.output_filename, "-{n}.png")));
            Some(c)
        },
    };

    if let Some(mut c) = command {
        let output = c.output().unwrap();
        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();
        if !output.status.success() {
            std::process::exit(-2);
        }
        io::stderr().write_all(&output.stderr).unwrap();
    }
}