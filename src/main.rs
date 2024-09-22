use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use xml::{
    name::OwnedName,
    reader::{EventReader, XmlEvent},
};

struct PostData {
    pub contents: String,
    pub link: String,
    pub date: String,
    pub title: String,
    pub has_password: bool,
    pub cur_tag: String,
    pub tags: Vec<String>,
}
#[derive(PartialEq, Eq, Debug)]
enum XmlTagType {
    Irrelevant,
    Contents,
    Link,
    Date,
    Title,
    Tag,
    Password,
}

impl PostData {
    fn new() -> PostData {
        PostData {
            contents: String::new(),
            link: String::new(),
            date: String::new(),
            title: String::new(),
            has_password: false,
            cur_tag: String::new(),
            tags: vec![],
        }
    }
}

struct Stats {
    pub total_posts: u32,
    pub posts_emitted: u32,
    pub posts_skipped_because_of_password: u32,
}
impl Stats {
    fn new() -> Stats {
        Stats {
            total_posts: 0,
            posts_emitted: 0,
            posts_skipped_because_of_password: 0,
        }
    }
}

fn name_matches(name: &OwnedName, expected: &str) -> bool {
    // TODO consider namespace I guess?
    name.local_name.eq(expected)
}

fn read_characters(cur_tag_type: &XmlTagType, data: &str, cur_post_data: &mut PostData) {
    match cur_tag_type {
        XmlTagType::Title => {
            cur_post_data.title.push_str(data);
        }
        XmlTagType::Tag => {
            cur_post_data.cur_tag.push_str(data);
        }
        XmlTagType::Contents => {
            cur_post_data.contents.push_str(data);
        }
        XmlTagType::Link => {
            cur_post_data.link.push_str(data);
        }
        XmlTagType::Date => {
            cur_post_data.date.push_str(data);
        }
        XmlTagType::Password => {
            if !data.is_empty() {
                cur_post_data.has_password = true;
            }
        }
        XmlTagType::Irrelevant => {}
    }
}

fn emit_header(file: &mut File, posts: &[PostData], stats: &Stats) -> std::io::Result<()> {
    write!(file, "<!DOCTYPE html><html><head>\n")?;
    write!(file, "<style>\n")?;
    let css_file = File::open("style.css")?;
    let css_file = BufReader::new(css_file);
    // this is ugly
    let mut newline_array = [0u8];
    '\n'.encode_utf8(&mut newline_array);
    for line in css_file.lines() {
        file.write(line?.as_bytes())?;
        file.write(&newline_array)?;
    }
    write!(file, "</style></head><body>\n")?;
    write!(file, "<details><summary>Table of Contents</summary><ul>\n")?;
    let mut index = 0;
    for post in posts {
        write!(
            file,
            "<li><a id=\"toc-{}\" href=\"#post-{}\">{}</a></li>\n",
            index, index, post.title
        )?;
        index += 1;
    }
    write!(file, "</ul></details>\n")?;
    write!(file, "<details><summary>Stats</summary><ul>\n")?;
    write!(
        file,
        "<li>Posts in this file: {}</li>\n",
        stats.posts_emitted
    )?;
    write!(
        file,
        "<li>Posts skipped because of password: {}</li>\n",
        stats.posts_skipped_because_of_password
    )?;
    write!(
        file,
        "<li>Total posts in backup: {}</li>\n",
        stats.total_posts
    )?;
    write!(file, "</ul></details>\n")?;
    Ok(())
}

fn emit_post(file: &mut File, post_data: &PostData, index: u32) -> std::io::Result<()> {
    write!(file, "<div class=\"post\" id=\"post-{}\">\n", index)?;
    write!(
        file,
        "<div class=\"title\"><a href=\"{}\">{}</a></div>\n",
        post_data.link, post_data.title
    )?;
    write!(file, "<div class=\"date\">{}</div>\n", post_data.date)?;
    write!(
        file,
        "<div class=\"tags\">Tags: {}</div>\n",
        post_data.tags.join(", ")
    )?;
    write!(
        file,
        "<div class=\"contents\">{}</div>\n",
        post_data.contents
    )?;
    write!(
        file,
        "<a href=\"#toc-{}\">(back to table of contents)</a></div>\n",
        index
    )?;
    Ok(())
}
fn emit_footer(file: &mut File) -> std::io::Result<()> {
    write!(file, "</body></html>")?;
    Ok(())
}

// TODO options for including password protected posts - look for post_password
fn main() -> std::io::Result<()> {
    let file = File::open("wordpress.xml")?;
    let file = BufReader::new(file); // Buffering is important for performance
    let parser = EventReader::new(file);
    let mut in_item = false;
    let mut cur_post_data = PostData::new();
    let mut cur_tag_type = XmlTagType::Irrelevant;
    let mut posts_to_write = Vec::<PostData>::new();
    let mut stats = Stats::new();
    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name_matches(&name, "item") {
                    in_item = true;
                } else {
                    if in_item {
                        if name_matches(&name, "title") {
                            cur_tag_type = XmlTagType::Title;
                        } else if name_matches(&name, "category") {
                            cur_tag_type = XmlTagType::Tag;
                        } else if name_matches(&name, "encoded") {
                            cur_tag_type = XmlTagType::Contents;
                        } else if name_matches(&name, "link") {
                            cur_tag_type = XmlTagType::Link;
                        } else if name_matches(&name, "post_date") {
                            cur_tag_type = XmlTagType::Date;
                        } else if name_matches(&name, "post_password") {
                            cur_tag_type = XmlTagType::Password;
                        }
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name_matches(&name, "item") {
                    in_item = false;
                    stats.total_posts += 1;
                    // TODO parameterize the tag name
                    if cur_post_data.tags.contains(&("books".to_string())) {
                        if cur_post_data.has_password {
                            stats.posts_skipped_because_of_password += 1;
                        } else {
                            stats.posts_emitted += 1;
                            posts_to_write.push(cur_post_data);
                        }
                    }
                    cur_post_data = PostData::new();
                    cur_tag_type = XmlTagType::Irrelevant;
                } else {
                    if in_item {
                        if name_matches(&name, "title") {
                            assert_eq!(XmlTagType::Title, cur_tag_type);
                            cur_tag_type = XmlTagType::Irrelevant;
                        } else if name_matches(&name, "encoded") {
                            assert_eq!(XmlTagType::Contents, cur_tag_type);
                            cur_tag_type = XmlTagType::Irrelevant;
                        } else if name_matches(&name, "link") {
                            assert_eq!(XmlTagType::Link, cur_tag_type);
                            cur_tag_type = XmlTagType::Irrelevant;
                        } else if name_matches(&name, "post_date") {
                            assert_eq!(XmlTagType::Date, cur_tag_type);
                            cur_tag_type = XmlTagType::Irrelevant;
                        } else if name_matches(&name, "post_password") {
                            assert_eq!(XmlTagType::Password, cur_tag_type);
                            cur_tag_type = XmlTagType::Irrelevant;
                        } else if name_matches(&name, "category") {
                            assert_eq!(XmlTagType::Tag, cur_tag_type);
                            if !cur_post_data.cur_tag.is_empty() {
                                cur_post_data.tags.push(cur_post_data.cur_tag.clone());
                                cur_post_data.cur_tag.clear();
                            }
                            cur_tag_type = XmlTagType::Irrelevant;
                        }
                    }
                }
            }
            Ok(XmlEvent::CData(data)) => {
                read_characters(&cur_tag_type, &data, &mut cur_post_data);
            }
            Ok(XmlEvent::Characters(data)) => {
                read_characters(&cur_tag_type, &data, &mut cur_post_data);
            }
            Err(e) => {
                panic!("Error: {e}");
            }
            // There's more: https://docs.rs/xml-rs/latest/xml/reader/enum.XmlEvent.html
            _ => {}
        }
    }

    let mut output_file = File::create("output.html")?;
    emit_header(&mut output_file, &posts_to_write, &stats)?;
    let mut index = 0;
    for post in posts_to_write.iter() {
        emit_post(&mut output_file, &post, index)?;
        index += 1;
    }

    emit_footer(&mut output_file)?;
    Ok(())
}
