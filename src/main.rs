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
    pub matches_tag: bool,
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
            matches_tag: false,
        }
    }
    fn clear(&mut self) {
        self.contents.clear();
        self.link.clear();
        self.date.clear();
        self.title.clear();
        self.matches_tag = false;
    }
}

fn name_matches(name: &OwnedName, expected: &str) -> bool {
    // TODO consider namespace I guess?
    name.local_name.eq(expected)
}

// TODO options for including password protected posts
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
                        }
                    }
                }
            }
            Ok(XmlEvent::EndElement { name }) => {
                if name_matches(&name, "item") {
                    in_item = false;
                    // TODO write it out here if it's valid
                    print!("{}\n", cur_post_data.title);
                    cur_post_data.clear();
                    cur_tag_type = XmlTagType::Irrelevant;
                } else {
                    if in_item {
                        if name_matches(&name, "title") {
                            assert_eq!(XmlTagType::Title, cur_tag_type);
                            cur_tag_type = XmlTagType::Irrelevant;
                        }
                    }
                }
            }
            // TODO handle Characters too I guess
            Ok(XmlEvent::CData(data)) => {
                match cur_tag_type {
                    XmlTagType::Title => {
                        cur_post_data
                            .title
                            .insert_str(cur_post_data.title.len(), &data);
                    }
                    _ => {
                        //TODO
                    }
                }
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
