use chrono::NaiveDateTime;


pub trait Chapter {
    fn get_name(&self) -> String;
    fn download_to_folder(&self, path: &str) -> Result<(), ()>;
}

#[derive(strum_macros::Display)]
pub enum ComicStatus {
    Other = -1,
    Ongoing,
    Suspended,
    Completed,
}

pub struct ComicData {
    pub name: String,
    pub url: String,
    pub status: ComicStatus,
    pub last_update: Option<NaiveDateTime>,
    pub chapters: u32,
}

impl ComicData {
    fn new() -> Self {
        return Self {
            name: "".to_owned(),
            url: "".to_owned(),
            status: ComicStatus::Other,
            last_update: None,
            chapters: 0,
        };
    }
}

pub trait Comic {
    fn get_name(&self) -> String;
    fn get_data(&self) -> Result<ComicData, ()>;
    fn get_chapters(&self) -> Result<Vec<Box<dyn Chapter>>, ()>;
}

pub trait ComicScraper {
    fn get_name(&self) -> String;
        
    fn search_simple_comics(&self, name: &str) -> Result<Vec<Box<dyn Comic>>, ()>;
}

pub mod ww5_mangakakalot_tv;
