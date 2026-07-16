# 🌐 fsociety-relay

**A Zero-Knowledge Ephemeral Signaling & Fallback Data Transfer Server for fsociety**

`fsociety-relay` is a lightweight HTTP and WebSocket relay server written in Rust. It serves as the bridge for `fsociety` clients to connect and share data across the internet when direct P2P connections are blocked by NATs, firewalls, or routers.

---

## ⚡ Core Functions

1.  **WebSocket Connection Stitching (P2P Tunneling)**
    When two clients connect to `GET /ws/<room>`, the server upgrades both streams to WebSockets, removes them from the registry, and stitches their TCP streams directly. It acts as a passive pipe forwarding binary frames, enabling an encrypted E2EE tunnel between the clients.
2.  **HTTP Fallback Hosting**
    For clients utilizing fallback downloads (e.g. mobile web downloads via QR codes), the server provides basic file metadata registration, chunked file upload, and direct HTTPS downloading. All file payloads are kept purely in-memory and zeroed out upon session close.

---

## 🛠️ API Reference

### 1. WebSocket Signaling
```
GET /ws/<room_code>
```
*   **Description**: Upgrade connection to WebSocket. 
*   **Behavior**: If the client is the first to connect, it is held in memory. When the second client connects, the server spawns a thread to pipe and forward binary messages back and forth between the two sockets.

### 2. HTTP Fallback Endpoints

#### File Registration
```
GET /register?room=<room_code>&portal=<file_hash>&file=<filename>&size=<size_in_bytes>
```
*   **Description**: Registers a file's metadata inside a specific room.
*   **Response**: `200 OK` (plain text `"Registered"`) or `400 Bad Request`.

#### File Upload
```
POST /upload/<room_code>/<file_hash>
Content-Type: application/octet-stream
```
*   **Description**: Uploads file payload binary data directly to the server's in-memory hashmap.
*   **Response**: `200 OK` (plain text `"Uploaded"`).

#### File Download
```
GET /download/<room_code>/<file_hash>
```
*   **Description**: Streams the file attachment payload back to the client.
*   **Response**: `200 OK` with headers `Content-Disposition: attachment; filename="<filename>"` and file data.

#### Room Web Page
```
GET /room/<room_code>
```
*   **Description**: Serves a lightweight HTML page listing all registered files in the room with direct download links. (Great for mobile browser download fallbacks).

#### Query Room Status
```
GET /join?room=<room_code>
```
*   **Description**: Returns plain text metadata of all files registered in the specified room.

---

## 🚀 Deployment Guide

### Option 1: Deploy to Render.com (Easiest)
This project is pre-configured with a blueprint for Render:
1.  Fork the repository.
2.  Log into Render Dashboard.
3.  Click **New +** > **Blueprint**.
4.  Select your repository and apply. Render will automatically build the service named `fsociety-relay` based on the [render.yaml](/render.yaml) file.

**Manual Web Service Configuration on Render:**
*   **Runtime**: `Rust`
*   **Root Directory**: `render-server`
*   **Build Command**: `cargo build --release`
*   **Start Command**: `./target/release/render-server`

---

### Option 2: Deploy to VPS or Private Server
To run `fsociety-relay` on your own Linux server:

1.  **Clone the code & build**:
    ```bash
    git clone https://github.com/YOUR_USERNAME/fsociety.git
    cd fsociety/render-server
    cargo build --release
    ```
2.  **Run the binary**:
    ```bash
    PORT=8080 ./target/release/render-server
    ```

#### Systemd Service configuration (For production)
Create a file at `/etc/systemd/system/fsociety-relay.service`:
```ini
[Unit]
Description=fsociety Relay Server
After=network.target

[Service]
Type=simple
User=nobody
Restart=always
WorkingDirectory=/opt/fsociety/render-server
Environment=PORT=10000
ExecStart=/opt/fsociety/render-server/target/release/render-server

[Install]
WantedBy=multi-user.target
```
Enable and start the service:
```bash
sudo systemctl enable fsociety-relay
sudo systemctl start fsociety-relay
```

---

## 🏗️ Architecture & Internals

### State Management
*   **State Structure**: `Arc<Mutex<HashMap<String, Room>>>`
*   **Tunnels**: `Arc<Mutex<HashMap<String, WebSocket<TcpStream>>>>`
*   State is thread-safe and shared across connection-handler threads. Tunnels are established and cleaned up automatically when socket connections close.

### Concurrency
*   Spawns a native OS thread for every incoming client stream.
*   Once a WebSocket stitching tunnel is formed, the handler thread switches to low-latency non-blocking read/write loops, maximizing resource efficiency.

### Security
*   **Zero-Knowledge**: The relay has no visibility into the decryption keys. Since the cryptographic handshake (X25519) occurs exclusively between the two end clients, the relay only sees ciphertext.
*   **Ephemeral Data**: No database is used. Files uploaded via the HTTP fallback are held in RAM and discarded immediately upon application restart or user connection closure.
