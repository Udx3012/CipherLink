use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;



#[derive(Clone, Debug)]
struct Room {
    files: Vec<FileEntry>,
    uploaded_files: HashMap<String, Vec<u8>>,
}

#[derive(Clone, Debug)]
struct FileEntry {
    name: String,
    size: String,
    hash: String,
}

type Rooms = Arc<Mutex<HashMap<String, Room>>>;

struct PrefixedStream {
    prefix: std::io::Cursor<Vec<u8>>,
    inner: TcpStream,
}

impl Read for PrefixedStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.prefix.read(buf)?;
        if n == 0 {
            self.inner.read(buf)
        } else {
            Ok(n)
        }
    }
}

impl Write for PrefixedStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl PrefixedStream {
    fn set_nonblocking(&self, nonblocking: bool) -> std::io::Result<()> {
        self.inner.set_nonblocking(nonblocking)
    }
}

use tungstenite::protocol::WebSocket;
type WsRooms = Arc<Mutex<HashMap<String, WebSocket<PrefixedStream>>>>;

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "10000".to_string());
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let ws_rooms: WsRooms = Arc::new(Mutex::new(HashMap::new()));
    
    println!("CipherLink Relay Server running on 0.0.0.0:{}", port);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            let rooms_clone = Arc::clone(&rooms);
            let ws_rooms_clone = Arc::clone(&ws_rooms);
            thread::spawn(move || {
                let _ = handle_http_client(stream, rooms_clone, ws_rooms_clone);
            });
        }
    }
    Ok(())
}

fn handle_http_client(mut stream: TcpStream, rooms: Rooms, ws_rooms: WsRooms) -> std::io::Result<()> {
    if let Ok(addr) = stream.peer_addr() { log_connection(&addr.to_string()); }
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() { return Ok(()); }
    let parts: Vec<&str> = lines[0].split_whitespace().collect();
    if parts.len() < 2 { return Ok(()); }
    let (method, path) = (parts[0], parts[1]);
    
    // Check if it's a WebSocket upgrade request (typically GET /ws/...)
    if method == "GET" && path.starts_with("/ws/") {
        let prefixed = PrefixedStream {
            prefix: std::io::Cursor::new(buffer[..bytes_read].to_vec()),
            inner: stream,
        };
        return handle_websocket(prefixed, path, ws_rooms);
    }
    
    match (method, path) {
        ("GET", p) if p.starts_with("/register") => handle_register(stream, p, rooms),
        ("GET", p) if p.starts_with("/join") => handle_join(stream, p, rooms),
        ("GET", p) if p.starts_with("/room/") => handle_room_web(stream, p, rooms),
        ("GET", p) if p.starts_with("/download/") => handle_download(stream, p, rooms),
        ("POST", p) if p.starts_with("/upload/") => handle_upload(stream, p, &request, rooms),
        _ => send_response(stream, "404 Not Found", "text/html", "<html><body><h1>404 Not Found</h1></body></html>"),
    }
}

fn handle_websocket(stream: PrefixedStream, path: &str, ws_rooms: WsRooms) -> std::io::Result<()> {
    let room_code = match path.strip_prefix("/ws/") {
        Some(code) => code.split('?').next().unwrap_or("").to_string(),
        None => return Ok(()),
    };
    if room_code.is_empty() { return Ok(()); }

    let mut ws = match tungstenite::accept(stream) {
        Ok(ws) => ws,
        Err(_) => return Ok(()),
    };

    let mut peer_ws = {
        let mut guard = ws_rooms.lock().unwrap();
        if let Some(existing_ws) = guard.remove(&room_code) {
            existing_ws
        } else {
            guard.insert(room_code, ws);
            return Ok(());
        }
    };

    let _ = ws.get_ref().set_nonblocking(true);
    let _ = peer_ws.get_ref().set_nonblocking(true);

    println!("Stitching WebSocket connections for room: {}", room_code);

    loop {
        let mut activity = false;

        match ws.read() {
            Ok(msg) => {
                if peer_ws.send(msg).is_err() { break; }
                activity = true;
            }
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }

        match peer_ws.read() {
            Ok(msg) => {
                if ws.send(msg).is_err() { break; }
                activity = true;
            }
            Err(tungstenite::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }

        if !activity {
            thread::sleep(Duration::from_millis(10));
        }
    }

    println!("WebSocket session closed for room: {}", room_code);
    Ok(())
}

fn handle_register(stream: TcpStream, path: &str, rooms: Rooms) -> std::io::Result<()> {
    if let Some(query) = path.split('?').nth(1) {
        let mut params = HashMap::new();
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=') { params.insert(key, value); }
        }
        if let (Some(&room), Some(&portal), Some(&file), Some(&size)) = 
            (params.get("room"), params.get("portal"), params.get("file"), params.get("size")) {
            if !validate_room_code(room) {
                return send_response(stream, "400 Bad Request", "text/plain", "Error: Invalid room code");
            }
            rooms.lock().unwrap().entry(room.to_string()).or_insert_with(|| Room { files: Vec::new(), uploaded_files: HashMap::new() })
                .files.push(FileEntry { name: file.to_string(), size: size.to_string(), hash: portal.to_string() });
            send_response(stream, "200 OK", "text/plain", "Registered")
        } else { send_response(stream, "400 Bad Request", "text/plain", "Error: Missing parameters") }
    } else { send_response(stream, "400 Bad Request", "text/plain", "Error: No parameters") }
}

fn handle_join(stream: TcpStream, path: &str, rooms: Rooms) -> std::io::Result<()> {
    if let Some(room_code) = path.split('?').nth(1).and_then(|q| q.strip_prefix("room=")) {
        let rooms = rooms.lock().unwrap();
        if let Some(room) = rooms.get(room_code) {
            let mut response = format!("Joined room: {}\n", room_code);
            for file in &room.files { response.push_str(&format!("FILE:{}:{}:{}\n", file.name, file.size, file.hash)); }
            send_response(stream, "200 OK", "text/plain", &response)
        } else { send_response(stream, "400 Bad Request", "text/plain", "Error: Room not found") }
    } else { send_response(stream, "400 Bad Request", "text/plain", "Error: No room specified") }
}

fn handle_upload(mut stream: TcpStream, path: &str, request: &str, rooms: Rooms) -> std::io::Result<()> {
    let path_parts: Vec<&str> = path.split('/').collect();
    if path_parts.len() >= 4 {
        let (room_code, file_hash) = (path_parts[2], path_parts[3]);
        let content_length = request.lines().find(|line| line.to_lowercase().starts_with("content-length:"))
            .and_then(|line| line.split(':').nth(1))
            .and_then(|len_str| len_str.trim().parse().ok()).unwrap_or(0);
        
        if content_length > 0 && request.find("\r\n\r\n").is_some() {
            let mut buffer = vec![0u8; content_length];
            let mut bytes_read = 0;
            while bytes_read < content_length {
                match stream.read(&mut buffer[bytes_read..]) {
                    Ok(0) => break,
                    Ok(n) => bytes_read += n,
                    Err(_) => break,
                }
            }
            
            let mut rooms = rooms.lock().unwrap();
            if let Some(room) = rooms.get_mut(room_code) {
                room.uploaded_files.insert(file_hash.to_string(), buffer);
                return send_response(stream, "200 OK", "text/plain", "Uploaded");
            }
        }
    }
    send_response(stream, "400 Bad Request", "text/plain", "Error: Upload failed")
}

fn handle_download(mut stream: TcpStream, path: &str, rooms: Rooms) -> std::io::Result<()> {
    let path_parts: Vec<&str> = path.split('/').collect();
    if path_parts.len() >= 4 {
        let (room_code, file_hash) = (path_parts[2], path_parts[3]);
        let rooms = rooms.lock().unwrap();
        if let Some(room) = rooms.get(room_code) {
            if let Some(file) = room.files.iter().find(|f| f.hash.starts_with(file_hash)) {
                if let Some(file_data) = room.uploaded_files.get(&file.hash) {
                    let header = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=\"{}\"\r\nContent-Length: {}\r\n\r\n", file.name, file_data.len());
                    stream.write_all(header.as_bytes())?;
                    return stream.write_all(file_data);
                }
            }
        }
    }
    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\nFile not found")
}

fn handle_room_web(mut stream: TcpStream, path: &str, rooms: Rooms) -> std::io::Result<()> {
    if let Some(room_code) = path.strip_prefix("/room/") {
        let rooms = rooms.lock().unwrap();
        let html = if let Some(room) = rooms.get(room_code) {
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Room {}</h1>{}</body></html>",
                room_code, room.files.iter().map(|f| format!("<div><h3>{}</h3><p>Size: {}</p><a href='/download/{}/{}'>Download</a></div>", f.name, f.size, room_code, f.hash)).collect::<Vec<_>>().join(""))
        } else {
            "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Room Not Found</h1></body></html>".to_string()
        };
        stream.write_all(html.as_bytes())
    } else {
        send_response(stream, "404 Not Found", "text/html", "<html><body><h1>404 Not Found</h1></body></html>")
    }
}

fn send_response(mut stream: TcpStream, status: &str, content_type: &str, body: &str) -> std::io::Result<()> {
    let response = format!("HTTP/1.1 {}\r\nContent-Type: {}\r\n\r\n{}", status, content_type, body);
    stream.write_all(response.as_bytes())
}

fn validate_room_code(room_code: &str) -> bool { room_code.len() == 6 && room_code.chars().all(|c| c.is_numeric()) }
fn log_connection(addr: &str) { println!("Connection from: {}", addr); }