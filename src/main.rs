use std::fs::read_dir;
use std::io::Read;
use std::{fs::File, path::Path, ffi::OsStr, fs, io::Write};
use clap::Parser;
use chrono::{DateTime, Utc, FixedOffset, NaiveDateTime};
use std::{io, env};

//pub mod scraper;

pub mod comic_scraper;

use crate::comic_scraper::*;
use crate::comic_scraper::ww5_mangakakalot_tv::*;

fn get_tmp_path() -> String {
    return "/tmp/awocadodl".into();
}

fn get_comic_path() -> String {
    let mut path: String = env::var("AWOKADO_DL_PATH").unwrap_or("".into());

    if path.is_empty() {
        path = env::var("HOME").unwrap() + "/Comics";
    }

    return path;
}

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

// TODO Find simpler way to pass args from subcommand.
// TODO add progress bar
fn download(comic_name: &String, get_first: bool, from_chapter: Option<usize>, to_chapter: Option<usize>, file_type: Option<String>) {
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

    let comic_path: String = get_comic_path();
    let tmp_path: String = get_tmp_path();
    let path: &String;

    let is_cbz: bool = file_type.clone().unwrap_or("".into()) == "cbz";

    path = if is_cbz {
        &tmp_path
    } else { 
        &comic_path
    };
  
    for n in _from_chapter.._to_chapter+1 {
        let chapter = &chapters[length - n];
        let partial_path: String = "/".to_owned() + comic.get_name().as_str() + "/" + chapter.get_name().as_str();


        println!("({}/{}) Downloading {} chapter - \"{}\"",
            n - _from_chapter + 1, _to_chapter - _from_chapter + 1,    
            n,  chapter.get_name());

        chapter.download_to_folder((path.clone() + partial_path.as_str()).as_str());
        
        if is_cbz {
            println!("({}/{}) Converting {} chapter - \"{}\"",
                n - _from_chapter + 1, _to_chapter - _from_chapter + 1,    
                n,  chapter.get_name());

            let mut buffer = Vec::new();

            let cbz_file = std::fs::File::create(comic_path.clone() + partial_path.as_str() + ".cbz").unwrap();
            let mut zip = zip::ZipWriter::new(cbz_file);

            for entry_res in read_dir((path.clone() + partial_path.as_str()).as_str()).unwrap() {
                let entry = entry_res.unwrap();

                if entry.file_type().unwrap().is_dir() {
                    continue;
                }

                let file_name_buf = entry.file_name();
                let file_name = file_name_buf.to_str().unwrap();

                zip.start_file_from_path(Path::new(file_name), zip::write::FileOptions::default()).unwrap();
                let mut f = File::open(entry.path().as_path()).unwrap();

                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&buffer).unwrap();
                buffer.clear();
            }

            zip.finish().unwrap();
        }
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
        Action::Download { from_chapter, to_chapter, format } 
            => download(&args.comic_name, args.get_first, from_chapter, to_chapter, format),
        _ => println!("Option \"{:?}\" don't exists", args.action)
    }
}
