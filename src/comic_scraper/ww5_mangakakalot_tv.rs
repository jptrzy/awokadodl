use std::fs::File;

use chrono::NaiveDateTime;

use super::{ComicScraper, Chapter, Comic, ComicStatus, ComicData};

/*
id: W5M
 */

pub fn trim_whitespace(s: &str) -> String {
    let mut new_str = s.trim().to_owned();
    let mut prev = ' '; // The initial value doesn't really matter
    new_str.retain(|ch| {
        let result = ch != ' ' || prev != ' ';
        prev = ch;
        result
    });
    new_str
}

pub struct W5MChapter {
    name: String,
    url: String,
}

impl Chapter for W5MChapter {
    fn get_name(&self) -> String {
       return self.name.clone(); 
    }

    fn download_to_folder(&self, path: &str) -> Result<(), ()> {
        let mut images = Vec::new();
        
        {
            let response = reqwest::blocking::get(self.url.to_owned())
                .unwrap()
                .text()
                .unwrap();

            let document = scraper::Html::parse_document(&response);

            let images_selector = scraper::Selector::parse("img.img-loading").unwrap();

            document.select(&images_selector).for_each(|x| images.push(x.value().attr("data-src").unwrap().to_owned()));
        }
       

        std::fs::create_dir_all(path);

        for (i, image_url) in images.iter().enumerate() {

            let ext = image_url.split(".").last().to_owned().unwrap();
            let mut file = File::create(path.to_owned() + "/" + &i.to_string() + "." + ext).unwrap();

            let file_len = reqwest::blocking::get(image_url)
                .unwrap().copy_to(&mut file);
        }

        return Err(()); 
    } 
}

pub struct W5MComic {
    name: String,
    url: String,
}

impl Comic for W5MComic {
    fn get_name(&self) -> String {
       return self.name.clone(); 
    }

    fn get_data(&self) -> Result<super::ComicData, ()> {
        let mut data = ComicData::new();

        data.name = self.name.clone();
        data.url = self.url.clone();

        // Status
        let response = reqwest::blocking::get(self.url.to_owned())
            .unwrap()
            .text()
            .unwrap();

        let document = scraper::Html::parse_document(&response);
            
        let status_selector = scraper::Selector::parse(".manga-info-text li:nth-of-type(3)").unwrap();

        data.status = match document.select(&status_selector).next().unwrap().inner_html().as_ref() {
            "Status : Ongoing" => ComicStatus::Ongoing,
            "Status : Completed" => ComicStatus::Completed,
            _ => ComicStatus::Other
        };

        // Last Updated
        let date_selector = scraper::Selector::parse(".manga-info-text li:nth-of-type(4)").unwrap();
       
        let date_text = document.select(&date_selector).next().unwrap().inner_html();
        // May 25,2019 - 12:00 PM
        data.last_update = NaiveDateTime::parse_from_str(date_text.as_ref(),
            "Last updated : %b %d,%Y - %H:%M %p").ok(); 

        // Chapters
        data.chapters = match self.get_chapters() {
            Ok(ok) => ok.len() as u32,
            _ => 0
        };

        return Ok(data); 
    }

    fn get_chapters(&self) -> Result<Vec<Box<dyn Chapter>>, ()> {
        let mut chapters: Vec<Box<dyn Chapter>> = Vec::new();

        let response = reqwest::blocking::get(self.url.to_owned())
            .unwrap()
            .text()
            .unwrap();

        let document = scraper::Html::parse_document(&response);
            
        let chapters_selector = scraper::Selector::parse(".manga-info-chapter span a").unwrap();

        document.select(&chapters_selector).for_each(|x| chapters.push(Box::new(W5MChapter { 
            url: "https://ww5.mangakakalot.tv/".to_owned() + x.value().attr("href").unwrap(),
            name: trim_whitespace(x.inner_html().as_str()),
        })) );

        return Ok(chapters);
    }
}

pub struct W5MComicScraper {}

impl ComicScraper for W5MComicScraper {
    fn get_name(&self) -> String {
        return "ww5.mangakakalot.tv".to_owned();
    }

    fn search_simple_comics(&self, name: &str) -> Result<Vec<Box<dyn super::Comic>>, ()> {
        let mut comics: Vec<Box<dyn super::Comic>> = Vec::new();

        let response = reqwest::blocking::get(
            "https://ww5.mangakakalot.tv/search/".to_owned() + name
            )
            .unwrap()
            .text()
            .unwrap();

        let document = scraper::Html::parse_document(&response);

        let title_selector = scraper::Selector::parse("h3.story_name>a").unwrap();

        document.select(&title_selector).for_each(|x| comics.push(Box::new(W5MComic { 
            url: "https://ww5.mangakakalot.tv/".to_owned() + x.value().attr("href").unwrap(),
            name: x.inner_html(),
        })) );

        return Ok(comics);
    }
}


