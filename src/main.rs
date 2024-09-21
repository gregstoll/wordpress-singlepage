use std::fs::File;
use std::io::BufReader;

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

fn name_matches(name: &OwnedName, expected: &str) -> bool {
    // TODO consider namespace I guess?
    name.local_name.eq(expected)
}

fn append_to(target: &mut String, to_append: &str) {
    target.insert_str(target.len(), to_append);
}
fn read_characters(cur_tag_type: &XmlTagType, data: &str, cur_post_data: &mut PostData) {
    match cur_tag_type {
        XmlTagType::Title => {
            // TODO - is there really no .append()?
            append_to(&mut cur_post_data.title, &data);
        }
        XmlTagType::Tag => {
            append_to(&mut cur_post_data.cur_tag, &data);
        }
        XmlTagType::Contents => {
            append_to(&mut cur_post_data.contents, &data);
        }
        XmlTagType::Link => {
            append_to(&mut cur_post_data.link, &data);
        }
        XmlTagType::Date => {
            append_to(&mut cur_post_data.date, &data);
        }
        XmlTagType::Password => {
            if !data.is_empty() {
                cur_post_data.has_password = true;
            }
        }
        XmlTagType::Irrelevant => {}
    }
}

// TODO options for including password protected posts - look for post_password
fn main() -> std::io::Result<()> {
    let file = File::open("wordpress.xml")?;
    let file = BufReader::new(file); // Buffering is important for performance

    let parser = EventReader::new(file);
    let mut in_item = false;
    let mut cur_post_data = PostData::new();
    let mut cur_tag_type = XmlTagType::Irrelevant;
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
                    // TODO write it out here if it's valid
                    // TODO parameterize the tag name
                    if cur_post_data.tags.contains(&("books".to_string()))
                        && !cur_post_data.has_password
                    {
                        print!("{}\n", cur_post_data.title);
                        //print!("{}\n\n", cur_post_data.contents);
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
    Ok(())
}
