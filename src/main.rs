extern crate ansi_term;
extern crate rustc_serialize;
extern crate hyper;
extern crate regex;

use ansi_term::Colour::*;
use rustc_serialize::json;
use hyper::client::*;
use std::io::*;
use regex::Regex;
//use std::process::Command;
use std::thread;
use std::sync::mpsc;
//use std::option::Option;
use std::time;

mod flickr;

fn main() {
    println!("{}", Yellow.bold().paint("CatCache"));

    let (tx_to_recharge, rx_to_recharge) = mpsc::channel();
    let (tx_to_buffer, rx_to_buffer) = mpsc::channel();
    let (tx_from_buffer, rx_from_buffer) = mpsc::channel();

    let tx_to_buffer2 = tx_to_buffer.clone();

    thread::spawn(move || {
        let mut photos_with_url_l: Vec<flickr::FlickrPhoto> = vec![];

        loop {
            match rx_to_buffer.recv().unwrap() {
                Some(p) => photos_with_url_l.push(p),
                None => {
                    if photos_with_url_l.is_empty() {
                        //tx_recharge.send("RECHARGE").unwrap();
                        tx_from_buffer.send(None).unwrap();
                        tx_to_recharge.send(200).unwrap();
                    } else {
                        let photo = photos_with_url_l.remove(0);
                        tx_from_buffer.send(Some(photo)).unwrap();
                        if photos_with_url_l.len() < 200 {
                            let wanted_n: u32 = 200 - (photos_with_url_l.len() as u32);
                            //tx_to_recharge.send(wanted_n).unwrap();
                            tx_to_recharge.send(wanted_n).unwrap();
                        }
                    }
                }
            }
        }
    });
    thread::spawn(move || {
        let mut pages_loaded = 0;
        loop {
            let wanted_n = rx_to_recharge.recv().unwrap();
            //let wanted_n: u32 = rx_to_recharge.recv().unwrap();
            //print!("[{}]", Cyan.bold().paint(format!("({})", wanted_n)));
            stdout().flush().unwrap();

            let mut photos_added = 0;
            while photos_added < wanted_n {
                print!("{}", Purple.bold().paint("?"));
                stdout().flush().unwrap();

                let cat_page = get_cat_page(pages_loaded);
                pages_loaded += 1;
                print!("\n{}", Purple.bold().paint(format!("({})", pages_loaded)));
                stdout().flush().unwrap();

                for photo in cat_page.photo {
                    if photo.url_l.is_some() {
                        tx_to_buffer2.send(Some(photo)).unwrap();
                        photos_added += 1;
                        print!("{}", Green.bold().paint("+"));
                    } else {
                        print!("{}", Red.bold().paint("."));
                    }
                    stdout().flush().unwrap();
                }
            }

            loop {
                match rx_to_recharge.try_recv() {
                    Ok(_) => {},
                    Err(_) => break,
                }
            }
        }
    });

    loop {
        tx_to_buffer.send(None).unwrap();
        match rx_from_buffer.recv().unwrap() {
            Some(_) => {
                print!("{}", Blue.bold().paint("-"));
                stdout().flush().unwrap();
            },
            None => continue
        }
        thread::sleep(time::Duration::from_millis(300));
    }
    /*
    let mut photos_with_url_l: Vec<flickr::FlickrPhoto> = vec![];

    let mut pages_loaded = 0;
    loop {
        while !photos_with_url_l.is_empty() {
            let photo = photos_with_url_l.remove(0);
            //println!("pages_loaded={} {}", pages_loaded, photo.title);
            //println!("  {}", photo.url_l.clone().unwrap());
            print!("{}", Blue.bold().paint("-"));
            stdout().flush().unwrap();

            let output = Command::new("wget")
                     .arg(photo.url_l.clone().unwrap())
                     .output()
                     .expect("failed to execute proces");
        }

        let cat_page = get_cat_page(pages_loaded);
        pages_loaded += 1;
        print!("\n{}", Purple.bold().paint(format!("({})", pages_loaded)));
        stdout().flush().unwrap();
        for photo in cat_page.photo {
            if photo.url_l.is_some() {
                print!("{}", Green.bold().paint("+"));
                photos_with_url_l.push(photo);
            } else {
                print!("{}", Red.bold().paint("."));
            }
            stdout().flush().unwrap();
        }
    }
    */
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
