#[derive(RustcDecodable, RustcEncodable)]
pub struct FlickrPhotosSearchResult {
    pub stat: String,
    pub photos: FlickrPhotosPage
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct FlickrPhotosPage {
    pub pages: u64,
    pub perpage: u64,
    pub total: u64,
    pub page: u64,
    pub photo: Vec<FlickrPhoto>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct FlickrPhoto {
    pub id: u64,
    pub owner: String,
    pub title: String,
    pub url_l: Option<String>,
}

/*
* When #buffer < desired_buffering, load a new page and its url_l photos to buffer.
* When an item is taken from the buffer,

* One thread to handle data access:
  * Provide a FlickrPhoto from the out of the FIFO buffer
  * Insert a FlickrPhoto to the end of the FIFO buffer
* One thread to handle requesting pages?
  * Or one thread per requested page?
* One webserver thread to serve images.

* The buffer can be an abstract FIFOBuffer.
* The data-from-flickr thread can be special purpose?
  * Called with message min_items=N, where N is how short the buffer is of items?
*/

/*
{
   "stat" : "ok",
   "photos" : {
      "pages" : 48252,
      "perpage" : 5,
      "total" : "241260",
      "page" : 1,
      "photo" : [
         {
            "farm" : 9,
            "isfamily" : 0,
            "title" : "Persie enjoying the Sun via http://ift.tt/29KELz0",
            "id" : "28604632606",
            "isfriend" : 0,
            "secret" : "695cce0a69",
            "owner" : "143919671@N07",
            "ispublic" : 1,
            "server" : "8676"
         },
         {
            "isfriend" : 0,
            "secret" : "48f446871d",
            "owner" : "53153433@N03",
            "server" : "8663",
            "ispublic" : 1,
            "height_l" : "1024",
            "farm" : 9,
            "title" : "2016-07-23%2010.17.35",
            "width_l" : "768",
            "isfamily" : 0,
            "id" : "28637076845",
            "url_l" : "https://farm9.staticflickr.com/8663/28637076845_48f446871d_b.jpg"
         },
         {
            "url_l" : "https://farm9.staticflickr.com/8709/28558031311_e6f8d16f7d_b.jpg",
            "id" : "28558031311",
            "title" : "Victor Chizhikov",
            "width_l" : "1024",
            "isfamily" : 0,
            "farm" : 9,
            "height_l" : "694",
            "ispublic" : 1,
            "server" : "8709",
            "owner" : "88526197@N00",
            "isfriend" : 0,
            "secret" : "e6f8d16f7d"
         },
         {
            "ispublic" : 1,
            "server" : "8897",
            "owner" : "88642461@N05",
            "secret" : "d437a67906",
            "isfriend" : 0,
            "url_l" : "https://farm9.staticflickr.com/8897/28527219282_d437a67906_b.jpg",
            "id" : "28527219282",
            "title" : "The Katzenjammer Band",
            "width_l" : "1024",
            "isfamily" : 0,
            "farm" : 9,
            "height_l" : "622"
         },
         {
            "isfriend" : 0,
            "secret" : "6f2282090d",
            "ispublic" : 1,
            "server" : "8657",
            "owner" : "93782583@N02",
            "farm" : 9,
            "height_l" : "1000",
            "url_l" : "https://farm9.staticflickr.com/8657/28530117112_6f2282090d_b.jpg",
            "id" : "28530117112",
            "title" : "Dovanojamas gyvūnas",
            "width_l" : "948",
            "isfamily" : 0
         }
      ]
   }
}
*/
