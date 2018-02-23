extern crate ansi_term;
extern crate hyper;
extern crate hyper_native_tls;
extern crate regex;
#[macro_use(chan_select)]
extern crate chan;
extern crate iron;
extern crate router;
extern crate rustc_serialize;
extern crate catcache;

use rustc_serialize::json;
use ansi_term::Colour::*;
use std::io::{stdout, Read, Write};
use regex::Regex;
use std::thread;
use std::sync::{Arc, Mutex};
use iron::status;
use iron::prelude::*;
use router::Router;
use std::option::Option;
use chan::{Receiver, Sender};
use hyper_native_tls::NativeTlsClient;
use hyper::net::HttpsConnector;
use iron::modifiers::Redirect;
use iron::Url;

use catcache::flickr::*;
use catcache::fifobuffer::*;

fn main() {
    println!("{}", Yellow.bold().paint("CatCache"));

    let (enqueue_tx, enqueue_rx) = chan::sync(10);
    let (refill_tx, refill_rx) = chan::async();
    let (dequeue_tx, dequeue_rx) = chan::sync(10);
    let (request_dequeue_tx, request_dequeue_rx) = chan::sync(10);

    let photo_buffer = Arc::new(Mutex::new(FIFOBuffer::new(200)));
    thread::spawn(move || {
                      run(photo_buffer,
                          enqueue_rx,
                          request_dequeue_rx,
                          dequeue_tx,
                          refill_tx);
                  });
    thread::spawn(move || { recharge(refill_rx, enqueue_tx); });

    let mut router = Router::new();
    router.get("/cat.jpg",
               move |_: &mut Request| {
        request_dequeue_tx.send(1);
        match dequeue_rx.recv().unwrap() {
            Some(cat_photo) => {
                let cat_url_l = cat_photo.url_l.unwrap();
                let url = Url::parse(&cat_url_l).unwrap();
                Ok(Response::with((status::TemporaryRedirect, Redirect(url))))
            }
            None => return Ok(Response::with((status::InternalServerError, ""))),
        }
    },
               "cat");

    Iron::new(router).http("localhost:3000").unwrap();
}

fn run<T>(buf: Arc<Mutex<FIFOBuffer<T>>>,
          enqueue_rx: Receiver<T>,
          request_dequeue_rx: Receiver<usize>,
          dequeue_tx: Sender<Option<T>>,
          refill_tx: Sender<usize>)
    where T: Sync,
          T: Send,
          T: 'static
{
    match buf.lock().unwrap().topup() {
        Some(n) => refill_tx.send(n),
        None => {}
    }

    loop {
        chan_select! {
            enqueue_rx.recv() -> item => {
                let i = item.unwrap();
                buf.lock().unwrap().push(i);
            },
            request_dequeue_rx.recv() -> desired_outs => {
                for _ in 0..desired_outs.unwrap() {
                    let option_item = buf.lock().unwrap().shift();
                    if option_item.is_some() {
                        print!("{}", Blue.bold().paint("-"));
                        stdout().flush().unwrap();
                    }
                    dequeue_tx.send(option_item);
                }

                match buf.lock().unwrap().topup() {
                    Some(n) => refill_tx.send(n),
                    None => {}
                }
            },
        }
    }
}

fn recharge(refill_rx: Receiver<usize>, enqueue_tx: Sender<FlickrPhoto>) {
    let mut pages_loaded = 0;
    let mut number_of_pages;
    loop {
        let wanted_n = refill_rx.recv().unwrap();
        let mut photos_added = 0;
        while photos_added < wanted_n {
            print!("{}", Purple.bold().paint("?"));
            stdout().flush().unwrap();

            let cat_page = get_cat_page(pages_loaded);
            number_of_pages = cat_page.pages;
            pages_loaded += 1;

            print!("\n{}",
                   Purple
                       .bold()
                       .paint(format!("({}/{})", pages_loaded, number_of_pages)));
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

            if pages_loaded >= number_of_pages {
                pages_loaded = 0;
                println!("{}", Purple.bold().paint("L"));
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

fn get_cat_page(page: u64) -> FlickrPhotosPage {
    let url: String = format!("https://api.flickr.\
                               com/services/rest/?api_key=6e8f097ad24b04e820faa21a96f9f6d7&method=flickr.\
                               photos.search&format=json&content_type=1&media=photos&extras=url_l&tags=cat&page={}&per_page=500",
                              page);
    let ssl = NativeTlsClient::new().unwrap();
    let connector = HttpsConnector::new(ssl);
    let client = hyper::Client::with_connector(connector);

    let mut response = match client.get(url.as_str()).send() {
        Ok(response) => response,
        Err(_) => {
            panic!("Whoops.");
        }
    };

    let mut buf = String::new();
    match response.read_to_string(&mut buf) {
        Ok(_) => (),
        Err(_) => panic!("I give up."),
    };

    let re = Regex::new(r"^jsonFlickrApi\(").unwrap();
    let re2 = Regex::new(r"\)$").unwrap();
    let buf2 = re.replace_all(buf.as_str(), "");
    let buf3: String = re2.replace_all(&buf2.into_owned(), "").into_owned();

    let photos_search_result: FlickrPhotosSearchResult = match json::decode(buf3.as_str()) {
        Ok(a) => a,
        Err(e) => panic!(e),
    };
    let photos_page = photos_search_result.photos;
    return photos_page;
}
