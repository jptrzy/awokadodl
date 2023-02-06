use std::{fs::File, path::Path, ffi::OsStr, fs};

use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime};

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

#[derive(strum_macros::Display)]
enum Status {
    Other = -1,
    Ongoing,
    Suspended,
    Completed,
}

struct ComicMoreInfo {
    status: Status,
    last_update: Option<NaiveDateTime>,
}

struct ComicShortInfo {
    url: String,
    name: String,
}

impl ComicShortInfo {
    fn get_more_info(&self) -> ComicMoreInfo {
        let mut more_info = ComicMoreInfo{
            status: Status::Other,
            last_update: None,
        };

        let response = reqwest::blocking::get(self.url.to_owned())
            .unwrap()
            .text()
            .unwrap();

        let document = scraper::Html::parse_document(&response);
            
        let status_selector = scraper::Selector::parse(".manga-info-text li:nth-of-type(3)").unwrap();

        more_info.status = match document.select(&status_selector).next().unwrap().inner_html().as_ref() {
            "Status : Ongoing" => Status::Ongoing,
            "Status : Completed" => Status::Completed,
            _ => Status::Other
        };
            
        let date_selector = scraper::Selector::parse(".manga-info-text li:nth-of-type(4)").unwrap();
       
        let date_text = document.select(&date_selector).next().unwrap().inner_html();
        // May 25,2019 - 12:00 PM
        more_info.last_update = NaiveDateTime::parse_from_str(date_text.as_ref(),
            "Last updated : %b %d,%Y - %H:%M %p").ok();
        return more_info;
    }

    fn get_chapters(&self) -> Vec<Chapter> {
        let mut chapters = Vec::new();

        let response = reqwest::blocking::get(self.url.to_owned())
            .unwrap()
            .text()
            .unwrap();

        let document = scraper::Html::parse_document(&response);
            
        let chapters_selector = scraper::Selector::parse(".manga-info-chapter span a").unwrap();

        document.select(&chapters_selector).for_each(|x| chapters.push(Chapter { 
            url: "https://ww5.mangakakalot.tv/".to_owned() + x.value().attr("href").unwrap(),
            name: trim_whitespace(x.inner_html().as_str()),
        }) );

        return chapters;
    }
}

struct Chapter {
    name: String,
    url: String,
}

impl Chapter {
    fn get_image_urls(&self) -> Vec<String> {
        let mut images = Vec::new();

        let response = reqwest::blocking::get(self.url.to_owned())
            .unwrap()
            .text()
            .unwrap();

        let document = scraper::Html::parse_document(&response);

        let images_selector = scraper::Selector::parse("img.img-loading").unwrap();

        document.select(&images_selector).for_each(|x| images.push(x.value().attr("data-src").unwrap().to_owned()));

        return images;
    }

    fn download_to_folder(&self, path: &str) {
        fs::create_dir_all(path);

        for (i, image_url) in self.get_image_urls().iter().enumerate() {

            let ext = image_url.split(".").last().to_owned().unwrap();
            let mut file = File::create(path.to_owned() + "/" + &i.to_string() + "." + ext).unwrap();

            let file_len = reqwest::blocking::get(image_url)
                .unwrap().copy_to(&mut file);
        }
    }
}

fn search_comic(name: &str) -> Vec<ComicShortInfo> {
    let mut comics = Vec::new(); 

    let response = reqwest::blocking::get(
        "https://ww5.mangakakalot.tv/search/".to_owned() + name
        )
        .unwrap()
        .text()
        .unwrap();

    let document = scraper::Html::parse_document(&response);

    let title_selector = scraper::Selector::parse("h3.story_name>a").unwrap();

    document.select(&title_selector).for_each(|x| comics.push(ComicShortInfo { 
        url: "https://ww5.mangakakalot.tv/".to_owned() + x.value().attr("href").unwrap(),
        name: x.inner_html(),
    }) );
    
    return comics;
}

fn main() {
    for comic in search_comic("Durarara") {
        println!("{}", comic.name);

        let data = comic.get_more_info();
        println!("{} {}", data.last_update.unwrap(), data.status);

        for chapter in comic.get_chapters() { 
            println!("{}", chapter.name);
    
            chapter.download_to_folder("out");
            break;
        }


        break;
    }
}
