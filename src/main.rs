use std::{fs::File, path::Path, ffi::OsStr, fs, io::Write};
use clap::Parser;
use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime};
use std::io;

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

#[derive(Clone, Debug)]
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

#[derive(Parser,Default,Debug)]
#[clap(author="jptrzy", version, about)]
/// A simple cli comic donwloader
struct Arguments {
    /// [info/search/download]
    option: String,
    
    /// If you don't specify a proper/full name,
    /// the program will still try to find one.
    #[clap(verbatim_doc_comment)]
    comic_name: String,
    
    /// Automaticly choose the fist comic in the list.
    #[clap(long, short='f', action)]
    get_first: bool,

    /// Where to start donwloading
    #[clap(long, short='s')]
    from_chapter: Option<usize>,
    /// Where to end downloading
    #[clap(long, short='e')]
    to_chapter: Option<usize>,

    /// Download format [crz/img]
    /// cbz - comic book archive
    /// img - image
    #[clap(long, short='o', verbatim_doc_comment)]
    format: Option<String>,
}

fn search_and_select_comic(comic_name: String, get_first: bool) -> Result<ComicShortInfo, String> {
    let comics = search_comic(comic_name.as_ref());

    if comics.is_empty() {
        return Err(format!("No comic was found by name \"{}\"", comic_name));
    }

    if get_first {
        return Ok(comics[0].clone());
    }

    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input="".to_owned();

        println!("q - Quit");

        for (i, comic) in comics.iter().enumerate() {
            println!("{} - {}", i, comic.name);
        }

        print!("$ ");
        io::stdout().flush();
        stdin.read_line(&mut input);

        if input.trim() == "q" {
            break;
        }

        let num: usize = match input.trim().parse() {
            Ok(ok) => ok,
            Err(_) => comics.len()
        };

        if num < comics.len() {
            return Ok(comics[num].clone()); 
        } else {
            println!("Can't recognize your input");
        } 
    }
    
    return Err("Quit".to_owned());
}

fn info(args: Arguments) {
    // println!("Start searching for \"{}\"", args.comic_name); 

    let comic = match search_and_select_comic(args.comic_name, args.get_first) {
        Ok(ok) => ok,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let more_info = comic.get_more_info(); 
    let chapters = comic.get_chapters().len();
    
    //  println!("Comic by the name \"{}\" was chosen", comic.name);

    println!("Name:         {}", comic.name);
    println!("Url:          {}", comic.url);
    println!("Status:       {}", more_info.status);
    
    if more_info.last_update.is_some() {
        println!("Last update:  {}", more_info.last_update.unwrap());
    } else {
        println!("Last update:  unknown");
    }

    println!("Chpters:      {}", chapters);
}

fn search(args: Arguments) {
    println!("Start searching for \"{}\"", args.comic_name); 

    let comics = search_comic(args.comic_name.as_str());

    if comics.is_empty() {
        println!("No comic was found by name \"{}\"", args.comic_name); 
    } else {
        println!("By this name thouse commics were found:");

        comics.iter().enumerate().for_each(
            |(i, comic)| println!("{} - {}", i, comic.name)
            );
    }
}

fn download(args: Arguments) {
    println!("Start searching for \"{}\"", args.comic_name); 

    let comic = match search_and_select_comic(args.comic_name, args.get_first) {
        Ok(ok) => ok,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let chapters = comic.get_chapters(); 

    let length = chapters.len();
    let from_chapter = args.from_chapter.unwrap_or(1);
    let to_chapter = args.to_chapter.unwrap_or(length);

    if 1 > from_chapter {
        println!("Can't start downloading chapter bellow 1"); 
        return;
    }
    
    if from_chapter > to_chapter || to_chapter > length {
        println!("Their isn't that much chapters to download or from_chapter is grater then to_chapter"); 
        return;
    }

    println!("Start downloading chapters {} to {}", from_chapter, to_chapter);

    for n in from_chapter..to_chapter+1 {

        let chapter = &chapters[length - n];

        println!("Downloading {} chapter that has name \"{}\"", n, chapter.name);

        chapter.download_to_folder(
            (comic.name.clone() + "/" + chapter.name.as_str()).as_str());
    }
}
 
fn main() {
    let args: Arguments = Arguments::parse();

    match args.option.as_str() {
        "info" => info(args),
        "search" => search(args),
        "download" => download(args),
        _ => println!("Option \"{}\" don't exists", args.option)
    }
}
