use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

static BIND_ADDR: &str = "127.0.0.1:8080";
static DOC_ROOT: &str = "C:\\www\\";
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

    fn parse_request(r: &str) -> Result<Box<HttpRequest>, HttpStatusCode> {
        if r.starts_with("GET") {
            let r = r.replace("\r\n", " ");
            let mut req_vec: Vec<&str> = r.split(' ').collect();
            for req in &mut req_vec {
                req.trim();
            }

            //TODO: URI handling and validation
            if req_vec.len() >= 3 && req_vec[1] == "/" && req_vec[2] == "HTTP/1.1" {
                return Ok(Box::new(HttpRequest::new(
                    HttpMethod::GET,
                    req_vec[1],
                    req_vec[2],
                    None,
                )));
            } else {
                return Err(HttpStatusCode::BadRequest);
            }
        } else if r.starts_with("POST") {
            return Err(HttpStatusCode::NotImplemented);
        } else if r.starts_with("UPDATE") {
            return Err(HttpStatusCode::NotImplemented);
        } else if r.starts_with("DELETE") {
            return Err(HttpStatusCode::NotImplemented);
        } else if r.starts_with("CONNECT") {
            return Err(HttpStatusCode::NotImplemented);
        } else if r.starts_with("TRACE") {
            return Err(HttpStatusCode::NotImplemented);
        } else if r.starts_with("HEAD") {
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

#[derive(Debug)]
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
    println!("Listening on {}", BIND_ADDR);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf: [u8; 1024] = [0; 1024];
    let bytes_read = stream.read(&mut buf).unwrap();
    let buf = String::from_utf8_lossy(&buf);

    let request = HttpRequest::parse_request(&buf).unwrap();

    match request.method {
        HttpMethod::GET => {
            println!(
                "GET request from {} -> \n{:?}",
                stream.peer_addr().unwrap(),
                request
            );

            let mut index_file = File::open("test.html").unwrap();
            let mut index_page = String::new();
            index_file.read_to_string(&mut index_page).unwrap();

            let response = format!("{} {} {}\r\n\r\n{}",HTTP_PROTO_VERSION, 
                                                        HttpStatusCode::HttpOk.value().0,
                                                        HttpStatusCode::HttpOk.value().1, 
                                                        index_page);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        _ => {
            //Should send an error code here for internal server error
            stream.flush().unwrap();
        }
    }
}