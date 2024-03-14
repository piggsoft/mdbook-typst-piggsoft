use std::fs::File;
use std::io;
use std::io::{Read, Write as IoWrite};
use std::path::PathBuf;

use mdbook::renderer::RenderContext;

mod to_typst;
mod config;
mod export;

use config::Config;

const TYPST_FILE_SUFFIX: &'static str = ".typ";

fn main() -> Result<(), anyhow::Error> {
    //env_logger::try_init_from_env(Env::default().default_filter_or("info"))?;

    let mut std_in = io::stdin();
    let context = RenderContext::from_json(&mut std_in)?;


    //let src_path = &context.root.join(&context.config.book.src);
    let dest_path = &context.root.join(&context.config.build.build_dir);

    let config = &context.config.get_deserialized_opt::<Config, &str>("output.typst-piggsoft")?.unwrap_or_default();

    let template_str = include_str!("assets/template.typ");

    let mut customer_template_str: Option<String> = None;

    if config.template_path.is_some() {
        customer_template_str = Some(read_template_str(&context.root.join(&config.template_path.as_ref().unwrap())));
    }

    let chapter_str = if customer_template_str.is_none() {
        to_typst::convert(&context, &config, &template_str)?
    } else {
        to_typst::convert(&context, &config, &customer_template_str.as_ref().unwrap())?
    };

    let file_name = format!("{}{}", &config.output_filename, TYPST_FILE_SUFFIX);

    let dest_typst_path = dest_path.join(&config.output_dir).join(file_name);

    // if !dest_path.exists() {
    //     std::fs::File::create(dest_path)?;
    // }

    let mut file = File::create(&dest_typst_path)?;

    let _ = &file.write_all(chapter_str.as_bytes())?;
    file.flush()?;

    export::export(&config, &context.root, &dest_typst_path, &dest_path.join(&config.output_dir));

    Ok(())
}

fn read_template_str(path: &PathBuf) -> String {
    let mut content = String::new();
    let mut file = File::open(path).expect("can not find the template file.");
    file.read_to_string(&mut content).expect("read template file error");
    content
}