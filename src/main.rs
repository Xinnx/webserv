use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

mod http;

use http::{HttpMethod, HttpRequest, HttpStatusCode};

static BIND_ADDR: &str = "127.0.0.1:8080";
static DOC_ROOT: &str = "C:/www";
static DEFAULT_INDEX: &str = "index.html";
static NOTFOUND_PAGE: &str = "404.html";
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
        Err(ref e) if *e == HttpStatusCode::BadRequest => {
            let response = format!(
                "{} {} {}\r\n\r\n 400 - BAD REQUEST",
                HTTP_PROTO_VERSION,
                HttpStatusCode::BadRequest.value().0,
                HttpStatusCode::BadRequest.value().1
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
        Err(ref e) if *e == HttpStatusCode::Unauthorized => {
            let response = format!(
                "{} {} {}\r\n\r\n 401 - UNAUTHORIZED",
                HTTP_PROTO_VERSION,
                HttpStatusCode::Unauthorized.value().0,
                HttpStatusCode::Unauthorized.value().1
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
        Err(ref e) if *e == HttpStatusCode::Forbidden => {
            let response = format!(
                "{} {} {}\r\n\r\n 403 - FORBIDDEN",
                HTTP_PROTO_VERSION,
                HttpStatusCode::Forbidden.value().0,
                HttpStatusCode::Forbidden.value().1
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
        Err(ref e) if *e == HttpStatusCode::NotFound => {
            let mut notfound_file = File::open(NOTFOUND_PAGE).unwrap();
            let mut notfound_page = String::new();
            notfound_file.read_to_string(&mut notfound_page).unwrap();

            let response = format!(
                "{} {} {}\r\n\r\n{}",
                HTTP_PROTO_VERSION,
                HttpStatusCode::NotFound.value().0,
                HttpStatusCode::NotFound.value().1,
                notfound_page
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
        Err(ref e) if *e == HttpStatusCode::NotImplemented => {
            let response = format!(
                "{} {} {}\r\n\r\n",
                HTTP_PROTO_VERSION,
                HttpStatusCode::NotImplemented.value().0,
                HttpStatusCode::NotImplemented.value().1
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
        Err(ref e) if *e == HttpStatusCode::InternalServerError => {}
        Err(ref e) => {
            let response = format!(
                "{} {} {}\r\n\r\n",
                HTTP_PROTO_VERSION,
                HttpStatusCode::BadRequest.value().0,
                HttpStatusCode::BadRequest.value().1
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
