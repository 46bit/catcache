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
use std::sync::{mpsc,Arc,Mutex};
//use std::option::Option;
use std::time;

mod flickr;

fn main() {
    println!("{}", Yellow.bold().paint("CatCache"));

    let (in_req_tx, in_req_rx) = mpsc::channel();
    let (in_tx, in_rx) = mpsc::channel();
    let (out_req_tx, out_req_rx) = mpsc::channel();
    let (out_tx, out_rx) = mpsc::channel();

    let photo_buffer = Arc::new(Mutex::new(FIFOBuffer::<flickr::FlickrPhoto>{
        items: vec![],
        desired_buffering: 200,
    }));
    let y = photo_buffer.clone();
    let z = photo_buffer.clone();

    thread::spawn(move || {
        run_in(y, in_rx);
    });
    thread::spawn(move || {
        run_out(z, out_req_rx, out_tx, in_req_tx);
    });
    thread::spawn(move || {
        recharge(in_req_rx, in_tx)
    });

    loop {
        out_req_tx.send(1).unwrap();
        match out_rx.recv().unwrap() {
            Some(p) => {
                //print!("[{}]", p.url_l.unwrap());
                //stdout().flush().unwrap();
            },
            None => continue
        }
        thread::sleep(time::Duration::from_millis(300));
    }
}

fn run_in(buf: Arc<Mutex<FIFOBuffer<flickr::FlickrPhoto>>>, chan_in: mpsc::Receiver<flickr::FlickrPhoto>) {
    loop {
        let item = chan_in.recv().unwrap();
        buf.lock().unwrap().push(item);
    }
}

fn run_out(buf: Arc<Mutex<FIFOBuffer<flickr::FlickrPhoto>>>, chan_out_req: mpsc::Receiver<usize>, chan_out: mpsc::Sender<Option<flickr::FlickrPhoto>>, chan_in_req: mpsc::Sender<usize>) {
    loop {
        match buf.lock().unwrap().topup() {
            Some(n) => chan_in_req.send(n).unwrap(),
            None => {}
        }

        let desired_outs = chan_out_req.recv().unwrap();
        for _ in 0..desired_outs {
            let mut option_item = None;
            if !buf.lock().unwrap().items.is_empty() {
                option_item = Some(buf.lock().unwrap().shift());
            }
            chan_out.send(option_item).unwrap();
        }
    }
}

struct FIFOBuffer<T> where T: std::marker::Sync, T: std::marker::Send {
    items: Vec<T>,
    desired_buffering: usize,
}

impl<T> FIFOBuffer<T> where T: std::marker::Sync, T: std::marker::Send {
    fn shift(&mut self) -> T {
        let item = self.items.remove(0);
        print!("{}", Blue.bold().paint("-"));
        stdout().flush().unwrap();
        item
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
        print!("{}", Green.bold().paint("+"));
        stdout().flush().unwrap();
    }

    fn topup(&mut self) -> Option<usize> {
        let items_len = self.items.len();
        if items_len < self.desired_buffering {
            return Some(self.desired_buffering - items_len)
        }
        None
    }
}

fn recharge(out_req: mpsc::Receiver<usize>, out: mpsc::Sender<flickr::FlickrPhoto>) {
    let mut pages_loaded = 0;
    loop {
        let wanted_n = out_req.recv().unwrap();
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
