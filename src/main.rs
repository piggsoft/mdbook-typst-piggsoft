use std::io;
use std::io::Write as IoWrite;

use env_logger::Env;
use mdbook::renderer::RenderContext;

use piggsoft::{export, to_typst};
use piggsoft::config::Config;

fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init_from_env(Env::default().default_filter_or("info"))?;

    let mut std_in = io::stdin();
    let context = RenderContext::from_json(&mut std_in)?;


    let template_str = include_str!("assets/template.typ");

    //let src_path = &context.root.join(&context.config.book.src);
    let dest_path = &context.root.join(&context.config.build.build_dir);

    let config = &context.config.get_deserialized_opt::<Config, &str>("output.typst-piggsoft")?.unwrap_or_default();

    let chapter_str = to_typst::convert(&context, &config, &template_str)?;


    let dest_typst_path = dest_path.join("typst-piggsoft/out.typ");

    // if !dest_path.exists() {
    //     std::fs::File::create(dest_path)?;
    // }

    let mut file = std::fs::File::create(&dest_typst_path)?;

    let _ = &file.write_all(chapter_str.as_bytes())?;
    file.flush()?;

    export::export(&config, &context.root, &dest_typst_path, &dest_path.join("typst-piggsoft"));

    Ok(())
}