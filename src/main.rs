use std::io;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use anyhow::anyhow;
use env_logger::Env;
use log::info;
use mdbook::book::Chapter;
use mdbook::BookItem;

use mdbook::renderer::RenderContext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub section_level: u32,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init_from_env(Env::default().default_filter_or("info"))?;

    let mut std_in = io::stdin();
    let context = RenderContext::from_json(&mut std_in)?;

    let config = context.config.get_deserialized_opt::<Config, &str>("output.typst-piggsoft")?.unwrap_or_default();

    let src_path = context.root.join(context.config.book.src);
    let dest_path = context.root.join(context.config.build.build_dir);

    info!("config is {:?}", config);

    for book_item in context.book.iter() {
        match book_item {
            &BookItem::Chapter(ref chapter) => {
                let chapter_path = &chapter.source_path.to_owned().ok_or(anyhow!("source_path not found"))?;


                let chapter_path = src_path.join(chapter_path);


                let chapter_parent_path = chapter_path.parent().ok_or(anyhow!("no parent"))?;


                ()
            }
            BookItem::Separator => {}
            BookItem::PartTitle(str) => {}
        }
    }


    Ok(())
}

fn join_file_path(from: PathBuf, to: PathBuf) -> PathBuf {
    let to_str = to.to_str().ok_or(anyhow!("no parent")).unwrap();
    if to_str.starts_with("http")
        || to_str.starts_with("https") {
        to.clone()
    } else {
        from.join(to)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::join_file_path;

    #[test]
    fn test_join_file_path() {
        let result = join_file_path(
            PathBuf::from(r#""D:\\Users\\piggsoft\\RustroverProjects\\piggsoft\\src"#),
            PathBuf::from(r#"https:://www.baidu.com"#)
        );
        assert_eq!(PathBuf::from(r#"https:://www.baidu.com"#), result);
    }
}

