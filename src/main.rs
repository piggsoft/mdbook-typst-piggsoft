use std::fmt::Write as FmtWrite;
use std::io;
use std::io::{Read, Write as IoWrite};
use std::ops::Deref;
use std::path::PathBuf;

use anyhow::anyhow;
use env_logger::Env;
use mdbook::BookItem;
use mdbook::renderer::RenderContext;
use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag};
use serde::{Deserialize, Serialize};

use crate::EventType::NoLN;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub section_level: u32,
    pub document_keywords: Option<String>,
}

enum EventType<'a> {
    ///  u32 is list level, none is first level
    List { level: usize, kind: ListKind },
    ///  u32 is list level, none is first level
    NumberedList(Option<u32>),
    FootnoteReference(CowStr<'a>),
    NoLN,
    TableHead,
    Heading,
}

enum ListKind {
    Numbered,
    UnSort,
}

fn main() -> Result<(), anyhow::Error> {
    env_logger::try_init_from_env(Env::default().default_filter_or("info"))?;

    let mut std_in = io::stdin();
    let context = RenderContext::from_json(&mut std_in)?;

    let config = context.config.get_deserialized_opt::<Config, &str>("output.typst-piggsoft")?.unwrap_or_default();

    let src_path = context.root.join(context.config.book.src);
    let dest_path = context.root.join(context.config.build.build_dir);


    let mut chapter_typst = String::new();

    let title = context.config.book.title.unwrap_or("Title".to_string());
    let authors = context.config.book.authors;
    let document_keywords = config.document_keywords.unwrap_or("Keywords".to_string());

    writeln!(chapter_typst, r#"#let document_title = "{}""#, title)?;
    writeln!(chapter_typst, r#"#let document_authors = "{}".split(",").map(it => it.trim())"#, authors.join(","))?;
    writeln!(chapter_typst, r#"#let document_keywords = "{}".split(",").map(it => it.trim())"#, document_keywords)?;


    let template_str = include_str!("assets/template.typ");


    chapter_typst.push_str(template_str);

    // let table_set = "#set table(fill: (col, row) => if calc.odd(row) { luma(240) } else { white })\n";
    // chapter_typst.push_str(table_set);
    // writeln!(chapter_typst, "#set pagebreak(weak: true)")?;
    //
    // writeln!(chapter_typst, r#"#outline(depth: 6, indent: 1em)"#)?;
    // writeln!(chapter_typst, r#"#pagebreak()"#)?;

    let mut event_stack: Vec<EventType> = Vec::new();

    for book_item in context.book.iter() {
        match book_item {
            BookItem::Chapter(chapter) => {
                let chapter_path = &chapter.source_path.to_owned().ok_or(anyhow!("source_path not found"))?;

                let chapter_path_str = chapter_path.to_str().ok_or(anyhow!("source_path not found"))?;
                let chapter_path_normal_str = chapter_path_str.replace("/", "_");
                let chapter_parent_path_str = chapter_path.parent().ok_or(anyhow!("no parent"))?.to_str().ok_or(anyhow!("source_path not found"))?;

                let options = Options::ENABLE_SMART_PUNCTUATION
                    | Options::ENABLE_STRIKETHROUGH
                    | Options::ENABLE_FOOTNOTES
                    | Options::ENABLE_TASKLISTS
                    | Options::ENABLE_TABLES;

                let parser = Parser::new_ext(&chapter.content, options);


                for event in parser {
                    match event {
                        Event::Start(tag) => {
                            match tag {
                                Tag::Paragraph => {
                                    //writeln!(chapter_typst, "#par()[")?
                                }
                                Tag::Heading(level, fragment_id, classes) => {
                                    let level_usize: usize = level as usize;
                                    write!(chapter_typst, "{} ", "=".repeat(level_usize))?;
                                    event_stack.push(EventType::Heading);
                                }
                                Tag::BlockQuote => {
                                    writeln!(chapter_typst, "#quote(block: true)[")?
                                }
                                Tag::CodeBlock(kind) => {
                                    match kind {
                                        CodeBlockKind::Indented => {
                                            write!(chapter_typst, "`")?;
                                        }
                                        CodeBlockKind::Fenced(language) => {
                                            if language.is_empty() {
                                                writeln!(chapter_typst, "```")?;
                                            } else {
                                                writeln!(chapter_typst, "```{}", language)?;
                                            }
                                        }
                                    }
                                }
                                Tag::List(index) => {
                                    let level = if let Some(EventType::List { level, kind }) = event_stack.last() {
                                        level + 1
                                    } else {
                                        0
                                    };

                                    if index.is_some() {
                                        event_stack.push(EventType::List { level, kind: ListKind::Numbered });
                                    } else {
                                        event_stack.push(EventType::List { level, kind: ListKind::UnSort });
                                    }
                                }
                                Tag::Item => {
                                    if let Some(EventType::List { level, kind }) = event_stack.last() {
                                        if let ListKind::Numbered = kind {
                                            write!(chapter_typst, "{}+ ", " ".repeat(*level))?;
                                        } else {
                                            write!(chapter_typst, "{}+ ", " ".repeat(*level))?;
                                        }
                                    } else {
                                        write!(chapter_typst, "- ")?;
                                    }
                                }
                                Tag::FootnoteDefinition(_) => {
                                    writeln!(chapter_typst, "#footnote[")?;
                                    event_stack.push(NoLN);
                                }
                                Tag::Table(aligns) => {
                                    let columns = aligns.len();
                                    let aligns = aligns.iter().map(|align| {
                                        match align {
                                            Alignment::None => String::from("auto"),
                                            Alignment::Left => String::from("left"),
                                            Alignment::Center => String::from("center"),
                                            Alignment::Right => String::from("right"),
                                        }
                                    }).reduce(|x, y| format!("{}, {}", x, y)).unwrap();
                                    writeln!(chapter_typst, "#table(columns: {}, align: ({}), ", columns, aligns)?;
                                }
                                Tag::TableHead => {
                                    event_stack.push(EventType::TableHead);
                                }
                                Tag::TableRow => {}
                                Tag::TableCell => {
                                    if let Some(EventType::TableHead) = event_stack.last() {
                                        write!(chapter_typst, "[*")?;
                                    } else {
                                        write!(chapter_typst, "[")?;
                                    }
                                    event_stack.push(EventType::NoLN);
                                }
                                Tag::Emphasis => {
                                    write!(chapter_typst, "_")?;
                                }
                                Tag::Strong => {
                                    write!(chapter_typst, "*")?;
                                }
                                Tag::Strikethrough => {
                                    write!(chapter_typst, "#strike[")?;
                                    event_stack.push(EventType::NoLN);
                                }
                                Tag::Link(link_type, url, title) => {
                                    match link_type {
                                        LinkType::Inline => {
                                            if url.starts_with("http") || url.starts_with("https") {
                                                write!(chapter_typst, r#"#link("{}")[{}"#, url, title)?;
                                            }
                                            if url.starts_with("#") {
                                                write!(chapter_typst, r#"#link(<{}-{}>)[{}"#, chapter_path_normal_str, url.replacen("#", "", 1), title)?;
                                            }
                                            if url.starts_with("mailto") {
                                                write!(chapter_typst, r#"#link("{}")[{}"#, url, title)?;
                                            }
                                            event_stack.push(NoLN);
                                        }
                                        LinkType::Reference => {}
                                        LinkType::ReferenceUnknown => {}
                                        LinkType::Collapsed => {}
                                        LinkType::CollapsedUnknown => {}
                                        LinkType::Shortcut => {}
                                        LinkType::ShortcutUnknown => {}
                                        LinkType::Autolink => {}
                                        LinkType::Email => {
                                            write!(chapter_typst, r#"#link("mailto:{}")[{}"#, url, title)?;
                                            event_stack.push(NoLN);
                                        }
                                    }
                                }
                                Tag::Image(link_type, url, title) => {
                                    match link_type {
                                        LinkType::Inline => {
                                            writeln!(chapter_typst, r#"#figure(image("../../src/{}/{}", alt: "{}"),)"#, chapter_parent_path_str, url, title)?;
                                        }
                                        LinkType::Reference => {}
                                        LinkType::ReferenceUnknown => {}
                                        LinkType::Collapsed => {}
                                        LinkType::CollapsedUnknown => {}
                                        LinkType::Shortcut => {}
                                        LinkType::ShortcutUnknown => {}
                                        LinkType::Autolink => {}
                                        LinkType::Email => {}
                                    }
                                }
                            }
                        }
                        Event::End(tag) => {
                            match tag {
                                Tag::Paragraph => {
                                    //writeln!(chapter_typst, "]")?;
                                    //writeln!(chapter_typst, "#parbreak()")?
                                    writeln!(chapter_typst, "")?;
                                }
                                Tag::Heading(_, _, _) => {
                                    //writeln!(chapter_typst, " <{}-{}>", chapter_path.to_str().ok_or(anyhow!("path error"))?, mdbook::utils::normalize_id())?;
                                }
                                Tag::BlockQuote => {
                                    writeln!(chapter_typst, "]")?;
                                }
                                Tag::CodeBlock(kind) => {
                                    match kind {
                                        CodeBlockKind::Indented => {
                                            write!(chapter_typst, "`")?;
                                        }
                                        CodeBlockKind::Fenced(_) => {
                                            writeln!(chapter_typst, "```")?;
                                        }
                                    }
                                }
                                Tag::List(_) => {
                                    event_stack.pop();
                                }
                                Tag::Item => {}
                                Tag::FootnoteDefinition(label) => {
                                    writeln!(chapter_typst, "] <{}> ", label)?;
                                }
                                Tag::Table(_) => {
                                    writeln!(chapter_typst, ")")?;
                                }
                                Tag::TableHead => {
                                    writeln!(chapter_typst, "")?;
                                    if let Some(EventType::TableHead) = event_stack.last() {
                                        event_stack.pop();
                                    }
                                }
                                Tag::TableRow => {
                                    writeln!(chapter_typst, "")?;
                                }
                                Tag::TableCell => {
                                    if let Some(EventType::TableHead) = event_stack.last() {
                                        write!(chapter_typst, "*], ")?;
                                    } else {
                                        write!(chapter_typst, "], ")?;
                                    }
                                }
                                Tag::Emphasis => {
                                    write!(chapter_typst, "_")?;
                                }
                                Tag::Strong => {
                                    write!(chapter_typst, "*")?;
                                }
                                Tag::Strikethrough => {
                                    write!(chapter_typst, "]")?;
                                }
                                Tag::Link(link_type, url, title) => {
                                    match link_type {
                                        LinkType::Inline => {
                                            writeln!(chapter_typst, "]")?
                                        }
                                        LinkType::Reference => {}
                                        LinkType::ReferenceUnknown => {}
                                        LinkType::Collapsed => {}
                                        LinkType::CollapsedUnknown => {}
                                        LinkType::Shortcut => {}
                                        LinkType::ShortcutUnknown => {}
                                        LinkType::Autolink => {}
                                        LinkType::Email => {
                                            writeln!(chapter_typst, "]")?
                                        }
                                    }
                                }
                                Tag::Image(_, _, _) => {}
                            }
                        }
                        Event::Text(text) => {
                            let text = text.replace("#", "\\#")
                                .replace("@", "\\@")
                                .replace("$", "\\$")
                                .replace("*", "\\*")
                                .replace("_", "\\_");

                            let event_type = event_stack.last();

                            match event_type {
                                Some(EventType::FootnoteReference(foot_note)) => {
                                    write!(chapter_typst, "{} @{} ", text, foot_note)?;
                                    event_stack.pop();
                                }
                                Some(EventType::NoLN) => {
                                    write!(chapter_typst, "{}", text)?;
                                    event_stack.pop();
                                }
                                Some(EventType::Heading) => {
                                    writeln!(chapter_typst, "{} <{}-{}>",
                                             text,
                                             chapter_path_normal_str,
                                             mdbook::utils::normalize_id(text.as_str())
                                    )?;
                                    event_stack.pop();
                                }
                                _ => {
                                    writeln!(chapter_typst, "{}", text)?;
                                }
                            }

                            //
                            // if let Some(EventType::FootnoteReference(foot_note)) = event_stack.last() {
                            //     writeln!(chapter_typst, "{} @{}", text, foot_note)?;
                            //     event_stack.pop();
                            // } else {
                            //     let text = text.replace("#", "\\#")
                            //         .replace("@", "\\@")
                            //         .replace("$", "\\$")
                            //         .replace("*", "\\*")
                            //         .replace("_", "\\_");
                            //     writeln!(chapter_typst, "{}", text)?;
                            // }
                        }
                        Event::Code(code) => {
                            write!(chapter_typst, "`{}`", code)?;
                        }
                        Event::Html(html) => {
                            write!(chapter_typst, "{}", html)?;
                        }
                        Event::FootnoteReference(foot_note) => {
                            event_stack.push(EventType::FootnoteReference(foot_note))
                        }
                        Event::SoftBreak => {}
                        Event::HardBreak => {}
                        Event::Rule => {}
                        Event::TaskListMarker(task_list) => {}
                    }
                }
            }
            BookItem::Separator => {}
            BookItem::PartTitle(str) => {}
        }
        writeln!(chapter_typst, "#pagebreak()")?;
    }

    let dest_path = dest_path.join("typst-piggsoft/out.typ");

    // if !dest_path.exists() {
    //     std::fs::File::create(dest_path)?;
    // }

    let mut file = std::fs::File::create(&dest_path)?;

    let _ = &file.write_all(chapter_typst.as_bytes())?;


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
            PathBuf::from(r#"https:://www.baidu.com"#),
        );
        assert_eq!(PathBuf::from(r#"https:://www.baidu.com"#), result);
    }

    #[test]
    fn test_join_file_path2() {
        let result = join_file_path(
            PathBuf::from(r#""D:\\Users\\piggsoft\\RustroverProjects\\piggsoft\\src"#),
            PathBuf::from(r#"/user/local"#),
        );
        assert_eq!(PathBuf::from(r#"/user/local"#), result);
    }

    #[test]
    fn test_join_file_path3() {
        let result = join_file_path(
            PathBuf::from(r#""D:\\Users\\piggsoft\\RustroverProjects\\piggsoft\\src"#),
            PathBuf::from(r#"C:\\Users\\xxx"#),
        );
        assert_eq!(PathBuf::from(r#"C:\\Users\\xxx"#), result);
    }
}