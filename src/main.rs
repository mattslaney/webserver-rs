use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream}, path::PathBuf,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut args = args.into_iter();

    let base_path_str = match args.find(|el| el == "--root") {
        Some(_) => args.next().expect("Expected base path to be supplied following --root argument"),
        None => "public".to_string()
    };
    let base_path = std::path::Path::new(&base_path_str).to_path_buf();
    
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind port");
    println!("Serving content from {:?} on http://127.0.0.1:7878", base_path);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(&base_path, stream);
    }
}

fn handle_connection(base_path: &std::path::PathBuf, mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = &buf_reader.lines().next().unwrap().unwrap();
    let request_parts: Vec<_> = request_line.split(' ').collect();
    let method = request_parts.get(0).unwrap();
    println!("Requested {:#?}", request_line);

    let mut file_path: PathBuf = std::path::PathBuf::new();
    let status: Status;
    let content: String;
    if *method == "GET" {
        let req_path = request_parts.get(1).unwrap();
        file_path = get_full_path(base_path, req_path);
        ( status, content ) = get_file_content_or_err(&file_path);
    } else {
        status = Status::ServerError;
        content = "Method unsupported".to_string();
    }

    println!("Responding with {:?} - {:?}", file_path, status);
    let response = format_response(status, content).to_owned();
    stream.write(response.as_bytes()).unwrap();
}

fn get_full_path(base_path: &std::path::PathBuf, path: &str) -> std::path::PathBuf {
    let mut path = match path {
        "/" => get_index_file(base_path),
        _ => path.to_string()
    };
    path = path.strip_prefix('/').unwrap().to_string();
    path = path.replace("../", "");
    base_path.join(path)
}

fn get_index_file(base_path: &std::path::PathBuf) -> String {
    let default_index_files = &[
        "index.html",
        "index.htm",
        "index.xhtml",
        "index.xml",
        "index.md",
        "index.txt",
    ];
    let file = default_index_files.iter().find(|path| {
        let full_path = std::path::Path::new(&base_path).join(&path);
        file_exists(&std::path::Path::new(&full_path).to_path_buf())
    });
    match file {
        Some(file) => format!("/{}", file),
        None => "/index.html".to_string()
    }
}

fn file_exists(file: &std::path::PathBuf) -> bool {
    let path = std::path::Path::new(file);
    path.exists()
}

fn get_file_content_or_err(file: &std::path::PathBuf) -> (Status, String) {
    match file_exists(&file) {
        true => {
            let content = std::fs::read_to_string(file);
            match content {
                Ok(content) => ( Status::Ok, content ),
                Err(_) => ( Status::ServerError, get_500() )
            }
        },
        false => {
            ( Status::NotFound, get_404() )
        }
    }
}

fn get_404() -> String {
    "Could not find requested file".to_string()
}

fn get_500() -> String {
    "An unexpected error has occurred".to_string()
}

#[derive(Debug)]
enum Status {
    Ok,
    NotFound,
    ServerError
}

fn format_response(status: Status, content: String) -> String {
    let status_line = match status {
        Status::Ok => "HTTP/1.1 200 OK",
        Status::NotFound => "HTTP/1.1 404 NOT FOUND",
        Status::ServerError => "HTTP/1.1 500 INTERNAL SERVER ERROR",
    };

    let length = content.len();

    format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}")
}
