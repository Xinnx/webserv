use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

static BIND_ADDR: &str = "127.0.0.1:8080";
static DOC_ROOT: &str = "C:/www";
static DEFAULT_INDEX: &str = "index.html";
static NOTFOUND_PAGE: &str = "404.html";
static HTTP_PROTO_VERSION: &str = "HTTP/1.1";

//TODO: prob should just make a HttpRequest structure with an option<T> for different types?
#[derive(Debug)]
pub struct HttpRequest<'a> {
    method: HttpMethod,
    req_uri: String,
    proto_ver: String,
    req_headers: Option<Box<HashMap<&'a str, &'a str>>>,
}

impl<'a> HttpRequest<'a> {
    fn new(
        method: HttpMethod,
        req_uri: &str,
        proto_ver: &str,
        req_headers: Option<Box<HashMap<&'a str, &'a str>>>,
    ) -> HttpRequest<'a> {
        HttpRequest {
            method,
            req_uri: String::from(req_uri),
            proto_ver: String::from(proto_ver),
            req_headers,
        }
    }

    fn parse_request(request: &str) -> Result<Box<HttpRequest>, HttpStatusCode> {
        let request = request.replace("\r\n", " ");
        let mut req_vec: Vec<&str> = request.split(' ').collect();
        for req in &mut req_vec {
            req.trim();
        }
        //TODO: URI handling and validation
        //Maybe this will stop directory transversal?
        println!("file: {:?}", &req_vec[1]);

        if request.starts_with("GET") {
            if req_vec[1] == "/" {
                req_vec[1] = DEFAULT_INDEX;
            }

            if req_vec[1].starts_with('/') && req_vec[1].len() > 1 {
                let mut s = req_vec[1];
                s = &s[1..];
                req_vec[1] = s;
            }

            let uri_path = PathBuf::from(&req_vec[1]).canonicalize();
            let uri_path = match uri_path {
                Ok(p) => p,
                Err(_) => return Err(HttpStatusCode::NotFound),
            };
            let doc_root_path = PathBuf::from(&DOC_ROOT).canonicalize().unwrap();

            println!(
                "Requested file: {:?} -> doc root: {:?}",
                &uri_path, &doc_root_path
            );
            if !uri_path.starts_with(&doc_root_path) {
                return Err(HttpStatusCode::BadRequest);
            }

            if req_vec.len() >= 3 && req_vec[2] == "HTTP/1.1" {
                return Ok(Box::new(HttpRequest::new(
                    HttpMethod::GET,
                    uri_path.to_str().unwrap(),
                    req_vec[2],
                    None,
                )));
            } else {
                return Err(HttpStatusCode::BadRequest);
            }
        } else if request.starts_with("POST") {
            return Err(HttpStatusCode::NotImplemented);
        } else if request.starts_with("UPDATE") {
            return Err(HttpStatusCode::NotImplemented);
        } else if request.starts_with("DELETE") {
            return Err(HttpStatusCode::NotImplemented);
        } else if request.starts_with("CONNECT") {
            return Err(HttpStatusCode::NotImplemented);
        } else if request.starts_with("TRACE") {
            return Err(HttpStatusCode::NotImplemented);
        } else if request.starts_with("HEAD") {
            return Err(HttpStatusCode::NotImplemented);
        } else if request.starts_with("OPTION") {
            return Err(HttpStatusCode::NotImplemented);
        } else {
            return Err(HttpStatusCode::BadRequest);
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
enum HttpMethod {
    GET,
    POST,
    UPDATE,
    DELETE,
    CONNECT,
    TRACE,
    HEAD,
    OPTION,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum HttpStatusCode {
    Continue,
    HttpOk,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    InternalServerError,
    NotImplemented,
}

impl HttpStatusCode {
    fn value(&self) -> (u16, &str) {
        match *self {
            HttpStatusCode::Continue => (100, "Continue"),
            HttpStatusCode::HttpOk => (200, "OK"),
            HttpStatusCode::BadRequest => (400, "Bad request"),
            HttpStatusCode::Unauthorized => (401, "Unauthorized"),
            HttpStatusCode::Forbidden => (403, "Forbidden"),
            HttpStatusCode::NotFound => (404, "Not found"),
            HttpStatusCode::InternalServerError => (500, "Internal server error"),
            HttpStatusCode::NotImplemented => (501, "Not implemented"),
        }
    }
}

fn main() {
    let listener = TcpListener::bind(BIND_ADDR).unwrap();
    println!(
        "Listening on {}
Default index: {}
Default 404 page: {}
Document root: {}",
        BIND_ADDR, DEFAULT_INDEX, NOTFOUND_PAGE, DOC_ROOT
    );

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
            _ => {
                let response = format!(
                    "{} {} {}\r\n\r\n",
                    HTTP_PROTO_VERSION,
                    HttpStatusCode::InternalServerError.value().0,
                    HttpStatusCode::InternalServerError.value().1
                );
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
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
            println!("{:?}", e);
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
            println!("{:?}", e);
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
            println!("{:?}", e);
        }
        Err(ref e) if *e == HttpStatusCode::InternalServerError => {}
        Err(e) => {
            let response = format!(
                "{} {} {}\r\n\r\n",
                HTTP_PROTO_VERSION,
                HttpStatusCode::BadRequest.value().0,
                HttpStatusCode::BadRequest.value().1
            );
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
            println!("{:?}", e)
        }
    };
}
