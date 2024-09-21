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
}

impl PostData {
    fn new() -> PostData {
        PostData {
            contents: String::new(),
            link: String::new(),
            date: String::new(),
            title: String::new(),
            cur_tag: String::new(),
            tags: vec![],
        }
    }
    fn clear(&mut self) {
        self.contents.clear();
        self.link.clear();
        self.date.clear();
        self.title.clear();
        self.cur_tag.clear();
        self.tags.clear();
    }
}

fn name_matches(name: &OwnedName, expected: &str) -> bool {
    // TODO consider namespace I guess?
    name.local_name.eq(expected)
}

fn read_characters(cur_tag_type: &XmlTagType, data: &str, cur_post_data: &mut PostData) {
    match cur_tag_type {
        XmlTagType::Title => {
            // TODO - is there really no .append()?
            cur_post_data
                .title
                .insert_str(cur_post_data.title.len(), &data);
        }
        XmlTagType::Tag => {
            cur_post_data
                .cur_tag
                .insert_str(cur_post_data.cur_tag.len(), &data);
        }
        _ => {
            //TODO
        }
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
                        }
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name_matches(&name, "item") {
                    in_item = false;
                    // TODO write it out here if it's valid
                    // TODO parameterize the tag name
                    if cur_post_data.tags.contains(&("books".to_string())) {
                        print!("{}\n", cur_post_data.title);
                    }
                    cur_post_data.clear();
                    cur_tag_type = XmlTagType::Irrelevant;
                } else {
                    if in_item {
                        if name_matches(&name, "title") {
                            assert_eq!(XmlTagType::Title, cur_tag_type);
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
