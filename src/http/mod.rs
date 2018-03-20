use std::collections::HashMap;
use std::path::{Path, PathBuf};

//TODO: prob should just make a HttpRequest structure with an option<T> for different types?
#[derive(Debug)]
pub struct HttpRequest<'a> {
    pub method: HttpMethod,
    pub req_uri: String,
    pub proto_ver: String,
    pub req_headers: Option<Box<HashMap<&'a str, &'a str>>>,
}

impl<'a> HttpRequest<'a> {
    pub fn new(
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

    //TODO: This needs to be refactored to only parse the http request and return the completed struct...valid or not
    //by returning an Err(_) we mask the original request and cannot get anymore information out of it later
    pub fn parse_request(request: &str) -> Result<Box<HttpRequest>, HttpStatusCode> {
        let request = request.replace("\r\n", " ");
        let mut req_vec: Vec<&str> = request.split(' ').collect();
        for req in &mut req_vec {
            req.trim();
        }

        //I am pretty sure all http requests have to specify at least the Method, URI, HTTP Protocol so the minimum length
        //for a valid request should be 3
        if req_vec.len() < 3 {
            return Err(HttpStatusCode::BadRequest);
        }
        if request.starts_with("GET") {

            //Requesting http://example.com would result in GET / HTTP/1.1
            //so we rewrite the request to the default index which is index.html -> GET index.html HTTP/1.1
            if req_vec[1] == "/" {
                req_vec[1] = ::DEFAULT_INDEX;
            }

            //Requesting http://example.com/afile.html would result in GET /afile.html HTTP/1.1
            //we just chop off the / here so when we canonicalize it it doesn't look at the root of the drive
            // ie /afile.html instead of ./afile.html
            if req_vec[1].starts_with('/') && req_vec[1].len() > 1 {
                let mut s = req_vec[1];
                s = &s[1..];
                req_vec[1] = s;
            }

            //Attempt to prevent directory recursion exploits hopfully and it has the added bonus
            //of checking if the file exists so we can return a 404
            let uri_path = PathBuf::from(&req_vec[1]).canonicalize();
            let uri_path = match uri_path {
                Ok(p) => p,
                Err(_) => return Err(HttpStatusCode::NotFound),
            };

            //Check if the (canonical)file is in the allowed doc root path
            let doc_root_path = PathBuf::from(&::DOC_ROOT).canonicalize().unwrap();
            if !uri_path.starts_with(&doc_root_path) {
                return Err(HttpStatusCode::BadRequest);
            }

            return Ok(Box::new(HttpRequest::new(
                HttpMethod::GET,
                uri_path.to_str().unwrap(),
                req_vec[2],
                None,
            )));
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

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
pub enum HttpMethod {
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
pub enum HttpStatusCode {
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
    pub fn value(&self) -> (u16, &str) {
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
