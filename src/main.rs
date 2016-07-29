extern crate ansi_term;
extern crate rustc_serialize;
extern crate hyper;
extern crate regex;

use ansi_term::Colour::*;
use rustc_serialize::json;
use hyper::client::*;
use std::io::Read;
use regex::Regex;

mod flickr;

fn main() {
    println!("{}", Yellow.bold().paint("CatCache"));

    let mut photos_with_url_l: Vec<flickr::FlickrPhoto> = vec![];

    let mut pages_loaded = 0;
    loop {
        while !photos_with_url_l.is_empty() {
            photos_with_url_l.remove(0);
            //println!("pages_loaded={} {}", pages_loaded, photo.title);
            //println!("  {}", photo.url_l.clone().unwrap());
            print!("{}", Blue.bold().paint("-"));
        }

        let cat_page = get_cat_page(pages_loaded);
        pages_loaded += 1;
        print!("\n{}", Purple.bold().paint(format!("({})", pages_loaded)));
        for photo in cat_page.photo {
            if photo.url_l.is_some() {
                print!("{}", Green.bold().paint("+"));
                photos_with_url_l.push(photo);
            } else {
                print!("{}", Red.bold().paint("."));
            }
        }
    }
}

fn get_cat_page(page: u32) -> flickr::FlickrPhotosPage {
    let url: String = format!("https://api.flickr.com/services/rest/?api_key=6e8f097ad24b04e820faa21a96f9f6d7&method=flickr.photos.search&format=json&content_type=1&media=photos&extras=url_l&tags=cats&page={}&per_page=500", page);

    let client = Client::new();
    let mut response = match client.get(&*url).send() {
        Ok(response) => response,
        Err(_) => panic!("Whoops."),
    };

    let mut buf = String::new();
    match response.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(_) => panic!("I give up."),
    };

    let re = Regex::new(r"^jsonFlickrApi\(").unwrap();
    let re2 = Regex::new(r"\)$").unwrap();
    let buf2 = re.replace_all(&*buf, "");
    let buf3: String = re2.replace_all(&*buf2, "");

    let photos_search_result: flickr::FlickrPhotosSearchResult = match json::decode(&*buf3) {
        Ok(a) => a,
        Err(e) => panic!(e),
    };
    let photos_page = photos_search_result.photos;
    return photos_page;
}
