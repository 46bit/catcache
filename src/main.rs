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

    let (in_req_tx, in_req_rx) = mpsc::channel();
    let (in_tx, in_rx) = mpsc::channel();
    let (out_req_tx, out_req_rx) = mpsc::channel();
    let (out_tx, out_rx) = mpsc::channel();

    let mut photo_buffer = FIFOBuffer::<flickr::FlickrPhoto>{
        items: vec![],
        desired_buffering: 200,
    };

    thread::spawn(move || {
        photo_buffer.run(in_req_tx, in_rx, out_req_rx, out_tx)
    });
    thread::spawn(move || recharge);

    loop {
        out_req_tx.send(1).unwrap();
        match out_rx.recv().unwrap() {
            Some(_) => {
                print!("{}", Blue.bold().paint("-"));
                stdout().flush().unwrap();
            },
            None => continue
        }
        thread::sleep(time::Duration::from_millis(300));
    }
}

struct FIFOBuffer<T> where T: std::marker::Sync, T: std::marker::Send {
    items: Vec<T>,
    desired_buffering: usize,
}

impl<T> FIFOBuffer<T> where T: std::marker::Sync, T: std::marker::Send {
    fn run(&self, chan_in_req: mpsc::Sender<usize>, chan_in: mpsc::Receiver<T>, chan_out_req: mpsc::Receiver<usize>, chan_out: mpsc::Sender<Option<T>>,) {
        let a = thread::spawn(move || {self.run_in(chan_in)});
        let b = thread::spawn(move || {self.run_out(chan_out_req, chan_out, chan_in_req)});
        a.join();
        b.join();
    }

    fn run_in(&self, chan_in: mpsc::Receiver<T>) {
        loop {
            let item = chan_in.recv().unwrap();
            self.push(item);
        }
    }

    fn run_out(&self, chan_out_req: mpsc::Receiver<usize>, chan_out: mpsc::Sender<Option<T>>, chan_in_req: mpsc::Sender<usize>) {
        loop {
            self.topup(chan_in_req);

            let desired_outs = chan_out_req.recv().unwrap();
            for i in 0..desired_outs {
                let mut option_item = None;
                if !self.items.is_empty() {
                    option_item = Some(self.shift());
                }
                chan_out.send(option_item).unwrap();
            }
        }
    }

    fn shift(&self) -> T {
        let item = self.items.remove(0);
        print!("{}", Blue.bold().paint("-"));
        stdout().flush().unwrap();
        item
    }

    fn push(&self, item: T) {
        self.items.push(item);
        print!("{}", Green.bold().paint("+"));
        stdout().flush().unwrap();
    }

    fn topup(&self, chan_in_req: mpsc::Sender<usize>) -> bool {
        let items_len = self.items.len();
        if items_len < self.desired_buffering {
            chan_in_req.send(self.desired_buffering - items_len).unwrap();
            return true
        }
        false
    }
}

fn recharge(minimum_items: u64, out_req: mpsc::Receiver<u32>, out: mpsc::Sender<flickr::FlickrPhoto>) {
    let mut pages_loaded = 0;
    loop {
        let wanted_n = out_req.recv().unwrap();
        //let wanted_n: u32 = rx_to_recharge.recv().unwrap();
        //print!("[{}]", Cyan.bold().paint(format!("({})", wanted_n)));
        stdout().flush().unwrap();

        let mut photos_added = 0;
        while photos_added < minimum_items {
            print!("{}", Purple.bold().paint("?"));
            stdout().flush().unwrap();

            let cat_page = get_cat_page(pages_loaded);
            pages_loaded += 1;
            print!("\n{}", Purple.bold().paint(format!("({})", pages_loaded)));
            stdout().flush().unwrap();

            for photo in cat_page.photo {
                if photo.url_l.is_some() {
                    out.send(photo).unwrap();
                    photos_added += 1;
                    print!("{}", Green.bold().paint("+"));
                } else {
                    print!("{}", Red.bold().paint("."));
                }
                stdout().flush().unwrap();
            }
        }

        loop {
            match out_req.try_recv() {
                Ok(_) => {},
                Err(_) => break,
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
