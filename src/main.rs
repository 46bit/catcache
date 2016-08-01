extern crate ansi_term;
extern crate rustc_serialize;
extern crate hyper;
extern crate regex;
#[macro_use(chan_select)]
extern crate chan;

use ansi_term::Colour::*;
use rustc_serialize::json;
use hyper::client::*;
use std::io::*;
use regex::Regex;
//use std::process::Command;
use std::thread;
use std::sync::{Arc,Mutex};
//use std::option::Option;
use std::time;
use std::collections::VecDeque;

mod flickr;

fn main() {
    println!("{}", Yellow.bold().paint("CatCache"));

    let (enqueue_tx, enqueue_rx) = chan::sync(10);
    let (refill_tx, refill_rx) = chan::async();
    let (dequeue_tx, dequeue_rx) = chan::sync(10);
    let (request_dequeue_tx, request_dequeue_rx) = chan::sync(10);

    let photo_buffer = Arc::new(Mutex::new(FIFOBuffer::<flickr::FlickrPhoto>{
        items: VecDeque::with_capacity(200),
        desired_buffering: 200,
    }));

    thread::spawn(move || {
        run(photo_buffer, enqueue_rx, request_dequeue_rx, dequeue_tx, refill_tx);
    });
    thread::spawn(move || {
        recharge(refill_rx, enqueue_tx);
    });

    loop {
        request_dequeue_tx.send(1);
        match dequeue_rx.recv().unwrap() {
            Some(_) => {
                //print!("[{}]", p.url_l.unwrap());
                //stdout().flush().unwrap();
            },
            None => continue
        }
        thread::sleep(time::Duration::from_millis(100));
    }
}

fn run(buf: Arc<Mutex<FIFOBuffer<flickr::FlickrPhoto>>>, enqueue_rx: chan::Receiver<flickr::FlickrPhoto>, request_dequeue_rx: chan::Receiver<usize>, dequeue_tx: chan::Sender<Option<flickr::FlickrPhoto>>, refill_tx: chan::Sender<usize>) {
    loop {
        match buf.lock().unwrap().topup() {
            Some(n) => refill_tx.send(n),
            None => {}
        }

        chan_select! {
            enqueue_rx.recv() -> item => {
                let i = item.unwrap();
                buf.lock().unwrap().push(i);
            },
            request_dequeue_rx.recv() -> desired_outs => {
                for _ in 0..desired_outs.unwrap() {
                    let mut option_item = None;
                    if !buf.lock().unwrap().items.is_empty() {
                        option_item = Some(buf.lock().unwrap().shift());
                    }
                    dequeue_tx.send(option_item);
                }
            },
        }
    }
}

struct FIFOBuffer<T> where T: std::marker::Sync, T: std::marker::Send {
    items: VecDeque<T>,
    desired_buffering: usize,
}

impl<T> FIFOBuffer<T> where T: std::marker::Sync, T: std::marker::Send {
    fn shift(&mut self) -> T {
        let item = self.items.pop_front().unwrap();
        print!("{}", Blue.bold().paint("-"));
        stdout().flush().unwrap();
        item
    }

    fn push(&mut self, item: T) {
        self.items.push_back(item);
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

fn recharge(refill_rx: chan::Receiver<usize>, enqueue_tx: chan::Sender<flickr::FlickrPhoto>) {
    let mut pages_loaded = 0;
    loop {
        let wanted_n = refill_rx.recv().unwrap();
        //let wanted_n: u32 = rx_to_recharge.recv().unwrap();
        //print!("[{}]", Cyan.bold().paint(format!("({})", wanted_n)));
        //stdout().flush().unwrap();

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
                    enqueue_tx.send(photo);
                    photos_added += 1;
                    print!("{}", Green.bold().paint("+"));
                } else {
                    print!("{}", Red.bold().paint("."));
                }
                stdout().flush().unwrap();
            }
        }

        loop {
            chan_select! {
                default => break,
                refill_rx.recv() => continue,
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
