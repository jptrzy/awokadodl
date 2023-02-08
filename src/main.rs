use std::{fs::File, path::Path, ffi::OsStr, fs, io::Write};
use clap::Parser;
use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime};
use std::io;

//pub mod scraper;

pub mod comic_scraper;

use crate::comic_scraper::*;
use crate::comic_scraper::ww5_mangakakalot_tv::*;

fn get_comic(comic_scraper: &dyn ComicScraper, comic_name: &str, get_first: bool) -> Result<Box<dyn Comic>, String> {
    let comics = match comic_scraper.search_simple_comics(comic_name) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error {:?}", err);
            return Err(("Unknown error while searching for comics".to_owned()));
        }
    };

    if comics.is_empty() {
        return Err(format!("No comic was found by name \"{}\"", comic_name));
    }

    if get_first {
        return Ok(comics.into_iter().nth(0).unwrap());
    }

    let stdin = std::io::stdin();
    let mut input = String::new();

    loop {
        input="".to_owned();

        println!("q - Quit");

        for (i, comic) in comics.iter().enumerate() {
            println!("{} - {}", i, comic.get_name());
        }

        print!("$ ");
        std::io::stdout().flush();
        stdin.read_line(&mut input);

        if input.trim() == "q" {
            break;
        }

        let num: usize = match input.trim().parse() {
            Ok(ok) => ok,
            Err(_) => comics.len()
        };

        if num < comics.len() {
            return Ok(comics.into_iter().nth(1).unwrap()); 
        } else {
            println!("Can't recognize your input");
        } 
    }
    
    return Err("Quit".to_owned());
}

fn info(args: Arguments) {
    let comic_scraper = W5MComicScraper{};
     
    let comic = match get_comic(&comic_scraper, &args.comic_name, args.get_first) {
        Ok(ok) => ok,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let data = match comic.get_data() {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error \"{:?}\" while serching for comic data.", err);
            return;
        }
    };

    println!("Name:         {}", data.name);
    println!("Url:          {}", data.url);
    println!("Status:       {}", data.status);
    
    if data.last_update.is_some() {
        println!("Last update:  {}", data.last_update.unwrap());
    } else {
        println!("Last update:  unknown");
    }

    println!("Chapters:     {}", data.chapters);
}

fn search(args: Arguments) {
    let comic_scraper = W5MComicScraper{};

    let comics = match comic_scraper.search_simple_comics(args.comic_name.as_str()) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error {:?}", err);
            return;
        }
    };

    if comics.is_empty() {
        println!("No comic was found by name \"{}\"", args.comic_name);
        return;
    } else {
        println!("By this name thouse commics were found:");

        comics.iter().enumerate().for_each(
            |(i, comic)| println!("{} - {}", i, comic.get_name())
            );
    }
}

fn download(comic_name: &String, get_first: bool, from_chapter: Option<usize>, to_chapter: Option<usize>, file_type: &Option<String>) {
    let comic_scraper = W5MComicScraper{};

    let comic = match get_comic(&comic_scraper, &comic_name, get_first) {
        Ok(ok) => ok,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let chapters = match comic.get_chapters() {
        Ok(ok) => ok,
        Err(err) => {
            println!("Unknown error while searching for chapters");
            return;
        }
    };

    let length = chapters.len();
    let _from_chapter = from_chapter.unwrap_or(1);
    let _to_chapter = to_chapter.unwrap_or(length);

    if 1 > _from_chapter {
        println!("Can't start downloading chapter bellow 1"); 
        return;
    }
    
    if _from_chapter > _to_chapter || _to_chapter > length {
        println!("Their isn't that much chapters to download or from_chapter is grater then to_chapter"); 
        return;
    }

    println!("Start downloading chapters {} to {}", _from_chapter, _to_chapter);

    for n in _from_chapter.._to_chapter+1 {

        let chapter = &chapters[length - n];

        println!("Downloading {} chapter that has name \"{}\"", n, chapter.get_name());

        chapter.download_to_folder(
            (comic.get_name() + "/" + chapter.get_name().as_str()).as_str());
    }
}

#[derive(clap::Parser, Debug)]
#[clap(author="jptrzy", version, about)]
/// A simple cli comic donwloader
struct Arguments {
    // Test
    #[clap(subcommand)]
    action: Action,
    
    /// If you don't specify a proper/full name,
    /// the program will still try to find one.
    #[clap(verbatim_doc_comment)]
    comic_name: String,
    
    /// Automaticly choose the fist comic in the list.
    #[clap(long, short='f', action)]
    get_first: bool,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
   Info,
   Search,
   Download {
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
   },
}

fn main() {
    let args: Arguments = Arguments::parse();
 
    match args.action {
        Action::Info => info(args),
        Action::Search => search(args),
        Action::Download { from_chapter, to_chapter, ref format } 
            => download(&args.comic_name, args.get_first, from_chapter, to_chapter, format),
        _ => println!("Option \"{:?}\" don't exists", args.action)
    }
}
