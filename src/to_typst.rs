use std::fmt::Write as FmtWrite;

use anyhow::anyhow;
use mdbook::BookItem;
use mdbook::renderer::RenderContext;
use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag};

use EventType::{TextNoNewLine, TextPostProcess};

use crate::config::Config;

enum EventType {
    ///  u32 is list level, none is first level
    List { level: usize, kind: ListKind },
    ///  u32 is list level, none is first level
    TableHead,
    TextNoNewLine,
    TextPostProcess,
}

enum ListKind {
    Numbered,
    UnSort,
}

fn map_reduce_table_aligns(aligns: &Vec<Alignment>) -> String {
    aligns.iter().map(|align| {
        match align {
            Alignment::None => String::from("auto"),
            Alignment::Left => String::from("left"),
            Alignment::Center => String::from("center"),
            Alignment::Right => String::from("right"),
        }
    }).reduce(|x, y| format!("{}, {}", x, y)).unwrap()
}

pub fn convert(context: &RenderContext, config: &Config, template_str: &str) -> Result<String, anyhow::Error> {
    let src_str = context.config.book.src.to_str().ok_or(anyhow!("src not found"))?;

    let mut chapter_str = String::new();
    let title = if let Some(title) = &context.config.book.title {
        title
    } else {
        "Title"
    };

    let authors = &context.config.book.authors;
    let document_keywords = if let Some(key_words) = &config.document_keywords {
        key_words
    } else {
        "Keywords"
    };
    let section_depth = &config.section_level;

    writeln!(chapter_str, r#"#let document_title = "{}""#, title)?;
    writeln!(chapter_str, r#"#let document_authors = "{}".split(",").map(it => it.trim())"#, authors.join(","))?;
    writeln!(chapter_str, r#"#let document_keywords = "{}".split(",").map(it => it.trim())"#, document_keywords)?;
    writeln!(chapter_str, r#"#let section_depth = {}"#, section_depth)?;


    chapter_str.push_str(template_str);

    // let table_set = "#set table(fill: (col, row) => if calc.odd(row) { luma(240) } else { white })\n";
    // chapter_typst.push_str(table_set);
    // writeln!(chapter_typst, "#set pagebreak(weak: true)")?;
    //
    // writeln!(chapter_typst, r#"#outline(depth: 6, indent: 1em)"#)?;
    // writeln!(chapter_typst, r#"#pagebreak()"#)?;

    for book_item in context.book.iter() {
        match book_item {
            BookItem::Chapter(ref chapter) => {
                let chapter_path = &chapter.source_path.to_owned().ok_or(anyhow!("source_path not found"))?;

                let chapter_path_str = chapter_path.to_str().ok_or(anyhow!("source_path not found"))?;
                let chapter_path_normal_str = chapter_path_str.replace("/", "_").replace(" ", "_");
                let chapter_parent_path_str = chapter_path.parent().ok_or(anyhow!("no parent"))?.to_str().ok_or(anyhow!("source_path not found"))?;


                let options = Options::ENABLE_SMART_PUNCTUATION
                    | Options::ENABLE_STRIKETHROUGH
                    | Options::ENABLE_FOOTNOTES
                    | Options::ENABLE_TASKLISTS
                    | Options::ENABLE_TABLES;

                let parser = Parser::new_ext(&chapter.content, options);

                let mut event_stack: Vec<EventType> = Vec::new();
                let mut content: CowStr = CowStr::from("");

                for event in parser {
                    match event {
                        Event::Start(Tag::Paragraph) => {}
                        Event::End(Tag::Paragraph) => { writeln!(chapter_str, "")? }
                        Event::Start(Tag::Heading(level, _, _)) => {
                            write!(chapter_str, "{} ", "=".repeat(level as usize))?;
                            event_stack.push(TextPostProcess);
                        }
                        Event::End(Tag::Heading(_, _, _)) => {
                            event_stack.pop();
                            writeln!(chapter_str, "{} <{}-{}>", content, chapter_path_normal_str, mdbook::utils::normalize_id(&content))?
                        }
                        Event::Start(Tag::BlockQuote) => { writeln!(chapter_str, "#quote(block: true)[")? }
                        Event::End(Tag::BlockQuote) => { writeln!(chapter_str, "]")? }
                        Event::Start(Tag::CodeBlock(kind)) => {
                            match kind {
                                CodeBlockKind::Indented => write!(chapter_str, "`")?,
                                CodeBlockKind::Fenced(language) => writeln!(chapter_str, "```{}", language)?,
                            }
                        }
                        Event::End(Tag::CodeBlock(kind)) => {
                            match kind {
                                CodeBlockKind::Indented => write!(chapter_str, "`")?,
                                CodeBlockKind::Fenced(_) => writeln!(chapter_str, "```")?,
                            }
                        }
                        Event::Start(Tag::List(index)) => {
                            let level = if let Some(EventType::List { level, kind: _ }) = event_stack.last() {
                                level + 1
                            } else {
                                0
                            };
                            if index.is_some() {
                                event_stack.push(EventType::List { level, kind: ListKind::Numbered })
                            } else {
                                event_stack.push(EventType::List { level, kind: ListKind::UnSort })
                            }
                        }
                        Event::Start(Tag::Item) => {
                            event_stack.push(TextPostProcess);
                        }
                        Event::End(Tag::Item) => {
                            event_stack.pop();
                            if let Some(EventType::List { level, kind }) = event_stack.last() {
                                if let ListKind::Numbered = kind {
                                    write!(chapter_str, "{}+ {}", " ".repeat(*level), &content)?;
                                } else {
                                    write!(chapter_str, "{}- {}", " ".repeat(*level), &content)?;
                                }
                            } else {
                                write!(chapter_str, "- {}", &content)?;
                            }
                        }
                        Event::End(Tag::List(_)) => { event_stack.pop(); }

                        Event::Start(Tag::FootnoteDefinition(_)) => { writeln!(chapter_str, "#footnote[")?; }
                        Event::End(Tag::FootnoteDefinition(label)) => { writeln!(chapter_str, "] <{}> ", label)?; }
                        Event::Start(Tag::Table(aligns)) => {
                            writeln!(chapter_str, "#table(columns: {}, align: ({}), ", aligns.len(), map_reduce_table_aligns(&aligns))?;
                        }
                        Event::Start(Tag::TableHead) => { event_stack.push(EventType::TableHead); }
                        Event::End(Tag::TableHead) => {
                            event_stack.pop();
                            writeln!(chapter_str, "")?;
                        }
                        Event::Start(Tag::TableRow) => {}
                        Event::Start(Tag::TableCell) => {
                            if let Some(EventType::TableHead) = event_stack.last() {
                                write!(chapter_str, "[*")?;
                            } else {
                                write!(chapter_str, "[")?;
                            }
                            event_stack.push(TextNoNewLine);
                        }
                        Event::End(Tag::TableCell) => {
                            if let Some(EventType::TableHead) = event_stack.last() {
                                write!(chapter_str, "*], ")?;
                            } else {
                                write!(chapter_str, "], ")?;
                            }
                        }
                        Event::End(Tag::TableRow) => { writeln!(chapter_str, "")?; }
                        Event::End(Tag::Table(_)) => { writeln!(chapter_str, ")")?; }

                        Event::Start(Tag::Emphasis) => {
                            write!(chapter_str, "_")?;
                            event_stack.push(TextNoNewLine);
                        }
                        Event::End(Tag::Emphasis) => {
                            write!(chapter_str, "_")?;
                        }
                        Event::Start(Tag::Strong) => {
                            write!(chapter_str, "*")?;
                            event_stack.push(TextNoNewLine);
                        }
                        Event::End(Tag::Strong) => {
                            write!(chapter_str, "*")?;
                        }
                        Event::Start(Tag::Strikethrough) => {
                            write!(chapter_str, "#strike[")?;
                            event_stack.push(TextNoNewLine);
                        }
                        Event::End(Tag::Strikethrough) => { write!(chapter_str, "]")?; }
                        Event::Start(Tag::Link(link_type, url, alt)) => {
                            match link_type {
                                LinkType::Inline => {
                                    if url.starts_with("http") || url.starts_with("https") {
                                        write!(chapter_str, r#"#link("{}")[{}"#, url, alt)?;
                                    } else if url.starts_with("#") {
                                        write!(chapter_str, r#"#link(<{}-{}>)[{}"#, chapter_path_normal_str, url.replacen("#", "", 1), alt)?;
                                    } else if url.starts_with("mailto") {
                                        write!(chapter_str, r#"#link("{}")[{}"#, url, alt)?;
                                    } else {
                                        write!(chapter_str, r#"#link("{}")[{}"#, url, alt)?;
                                    }
                                    event_stack.push(TextNoNewLine);
                                }
                                LinkType::Reference => {}
                                LinkType::ReferenceUnknown => {}
                                LinkType::Collapsed => {}
                                LinkType::CollapsedUnknown => {}
                                LinkType::Shortcut => {}
                                LinkType::ShortcutUnknown => {}
                                LinkType::Autolink => {
                                    write!(chapter_str, r#"#link("{}")[{}"#, url, alt)?;
                                }
                                LinkType::Email => {
                                    write!(chapter_str, r#"#link("mailto:{}")[{}"#, url, alt)?;
                                    event_stack.push(TextNoNewLine);
                                }
                            }
                        }
                        Event::End(Tag::Link(link_type, _, _)) => {
                            match link_type {
                                _ => {
                                    writeln!(chapter_str, "]")?
                                }
                            }
                        }
                        Event::Start(Tag::Image(link_type, url, alt)) => {
                            match link_type {
                                LinkType::Inline => {
                                    if "" == chapter_parent_path_str {
                                        write!(chapter_str, r#"#figure(image("/{}/{}", alt: "{}"), caption:["#, &src_str, url, alt)?;
                                    } else {
                                        write!(chapter_str, r#"#figure(image("/{}/{}/{}", alt: "{}"), caption:["#, &src_str, chapter_parent_path_str, url, alt)?;
                                    }
                                    event_stack.push(TextNoNewLine);
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
                        Event::End(Tag::Image(_, _, _)) => {
                            writeln!(chapter_str, "])")?;
                        }
                        Event::Text(text) => {
                            let text = text.replace("#", "\\#")
                                .replace("@", "\\@")
                                .replace("$", "\\$")
                                .replace("*", "\\*")
                                .replace("_", "\\_");
                            match event_stack.last() {
                                Some(EventType::TextPostProcess) => {
                                    content = CowStr::from(text);
                                }
                                Some(EventType::TextNoNewLine) => {
                                    event_stack.pop();
                                    write!(chapter_str, "{}", text)?;
                                }
                                Some(_) => { writeln!(chapter_str, "{}", text)?; }
                                None => { writeln!(chapter_str, "{}", text)?; }
                            }
                        }
                        Event::Code(_) => {}
                        Event::Html(_) => {}
                        Event::FootnoteReference(_) => {}
                        Event::SoftBreak => {}
                        Event::HardBreak => {}
                        Event::Rule => {}
                        Event::TaskListMarker(_) => {}
                    }
                }
            }
            BookItem::Separator => {}
            BookItem::PartTitle(_) => {}
        }
        writeln!(chapter_str)?;
        writeln!(chapter_str, "#pagebreak()")?;
    }

    Ok(chapter_str)
}
