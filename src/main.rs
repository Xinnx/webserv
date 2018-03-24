mod http;

use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

use http::{HttpMethod, HttpRequest, HttpStatusCode};

static BIND_ADDR: &str = "127.0.0.1:8080";
static DOC_ROOT: &str = "C:/www";
static DEFAULT_INDEX: &str = "index.html";
static NOTFOUND_PAGE: &str = "404.html";

//This project currently is referencing RFC 2616 for the implementation of HTTP/1.1, I wouldn't change this...
static HTTP_PROTO_VERSION: &str = "HTTP/1.1";

fn main() {
    let listener = TcpListener::bind(BIND_ADDR).unwrap();
    println!(
        "Listening on {}
Default index: {}
Default 404 page: {}
Document root: {}",
        BIND_ADDR, DEFAULT_INDEX, NOTFOUND_PAGE, DOC_ROOT
    );

    //TODO: add a thread pool for incoming connections.
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf: [u8; 1024] = [0; 1024];
    let _bytes_read = stream.read(&mut buf).unwrap();
    let buf = String::from_utf8_lossy(&buf);
    let request = HttpRequest::parse_request(&buf);
    match request {
        Ok(req) => match req.method {
            HttpMethod::GET => {
                println!(
                    "GET request from {} -> \n{:?}",
                    stream.peer_addr().unwrap(),
                    &req
                );

                let f = PathBuf::from(&req.req_uri);
                let mut index_file = File::open(&f).unwrap();
                let mut index_page = String::new();
                index_file.read_to_string(&mut index_page).unwrap();

                let response = format!(
                    "{} {} {}\r\n\r\n{}",
                    HTTP_PROTO_VERSION,
                    HttpStatusCode::HttpOk.value().0,
                    HttpStatusCode::HttpOk.value().1,
                    index_page
                );
                stream.write(response.as_bytes()).unwrap();

                stream.flush().unwrap();
            }
            HttpMethod::POST => {}
            HttpMethod::DELETE => {}
            HttpMethod::UPDATE => {}
            HttpMethod::HEAD => {}
            HttpMethod::OPTION => {}
            HttpMethod::CONNECT => {}
            HttpMethod::TRACE => {}
        },
        Err(ref e) => {
            let response = format!(
                "{} {} {}\r\n\r\n",
                HTTP_PROTO_VERSION,
                e.value().0,
                e.value().1
            );
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
            println!(
                "received {:?}  from {} -> {:?}",
                e,
                stream.peer_addr().unwrap().ip(),
                e
            );
        }
    };
}
