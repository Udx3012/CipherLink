# 💻 CipherLink

**Secure, Ephemeral, End-to-End Encrypted P2P Chat & File Streaming CLI**

[![Language](https://img.shields.io/badge/Language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Security](https://img.shields.io/badge/Security-E2EE%20(X25519%20%2B%20ChaCha20Poly1305)-green)](https://en.wikipedia.org/wiki/End-to-end_encryption)

`CipherLink` is a lightweight, terminal-based peer-to-peer (P2P) file sharing and instant chat application. Built in pure Rust, it uses end-to-end encryption (E2EE) to guarantee that your conversations and file streams are completely private. With support for both direct local connections and global internet tunnels via an un-trusted WebSocket relay server, `CipherLink` enables instant communication across networks without any cloud storage reliance.

## 🖼️ User Interface

![Terminal Setup](assets/terminal.png)

![Chat Session](assets/chat.png)

---

## ✨ Key Features

*   **🔒 Cryptographic E2EE (Zero-Trust)**: Uses an ephemeral X25519 Diffie-Hellman handshake to derive session keys on the fly, securing all communications with ChaCha20Poly1305 authenticated encryption.
*   **⚡ Zero-Upload File Streaming**: Files are sent in real-time. The receiver downloads chunks as the sender uploads them, bypassing slow cloud staging services.
*   **💾 Low Memory Overhead**: Files are streamed in 256KB chunks directly from disk. You can transfer large files without bloat or high RAM utilization.
*   **🌐 Flexible Networking**:
    *   **Global Mode**: Uses a custom WebSocket relay server to stitch connections across firewalls and NATs.
    *   **Local Mode**: Establishes direct TCP connections on local networks.
*   **🖥️ Cyberpunk TUI (Terminal User Interface)**: A beautiful green-on-black interface built using `ratatui` featuring split panels, system logs, active transfers, and an interactive input prompt.
*   **🔑 Custom Room codes**: Hosts can specify their own custom 6-digit numeric room code or let the client generate a secure, random one.

---

## 🛠️ Technical Architecture

```
                 [ Ephemeral X25519 DH Handshake ]
                              │
       ┌──────────────────────┴──────────────────────┐
   (Host Public Key)                             (Peer Public Key)
       ▼                                             ▼
[ Local Secret ] ◄─────────────────────────────► [ Local Secret ]
       │                                             │
       └──────────────► [ Shared Key ] ◄─────────────┘
                              │
               [ ChaCha20Poly1305 Encryption ]
                              │
                  ┌───────────┴───────────┐
                  ▼                       ▼
           [ Chat Message ]       [ 256KB File Chunk ]
```

1.  **Stitching & Signaling**: When staging a global connection, the clients connect to the relay server using WebSockets. The server matches the clients via a 6-digit room code and stitches their TCP/WebSocket streams together.
2.  **Handshake**: Once matched, the clients perform an ephemeral X25519 key exchange. They derive a 32-byte shared secret, hash it using SHA-256, and use it to initialize a symmetric `ChaCha20Poly1305` cipher.
3.  **Authentication Fingerprint**: A 4-character hex fingerprint (e.g. `A3D2-E409`) is computed from the public keys. Users can verbally verify this fingerprint to prevent Man-in-the-Middle (MitM) attacks.
4.  **Zero-Trust Relay**: The relay server only passes encrypted binary packets. Because it lacks access to the ephemeral private keys, it cannot decrypt any chat messages or file streams.

---

## 🚀 Quick Start

### Prerequisites
Make sure you have Rust and Cargo installed:
*   [Install Rust](https://rustup.rs/)

### Installation & Run

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/Udx3012/CipherLink.git
    cd CipherLink
    ```
2.  **Build the project**:
    ```bash
    cargo build --release
    ```

---

## 📖 Usage Guide

`CipherLink` can be run in two modes: **Global Mode** (via the Render relay server) or **Local Mode** (direct TCP connection over local networks).

### 1. Global Mode (Across the Internet)

**Step 1: Host stages the room**
The host starts the connection. You can optionally specify a custom 6-digit room code:
```bash
# Option A: Generate a random room code
cargo run --release stage

# Option B: Use a custom 6-digit room code
cargo run --release stage 133707
```
The terminal will display a room code and a QR code for mobile web fallback.

**Step 2: Friend joins the room**
Your friend connects to your room from a different location:
```bash
cargo run --release connect <6-DIGIT-CODE>
```

---

### 2. Local Mode (Direct Local Network)

Useful for high-speed local transfers when devices are on the same WiFi/Ethernet network.

**Step 1: Host starts the listener**
```bash
cargo run --release stage --local
```

**Step 2: Friend connects directly to host's IP**
```bash
cargo run --release connect --local <HOST-IP>:8766
```

---

## 💬 Chat Controls

Once you enter the secure interactive chat terminal:

*   **Standard Chat**: Type your message in the input prompt and press `Enter`.
*   **📁 Send a File**: Type `/send ` followed by the path to the file you want to share:
    ```text
    /send C:\Documents\archive.zip
    ```
    *The file is read in 256KB blocks, encrypted, and streamed to the peer. A progress bar will show on both screens.*
*   **🚪 Exit**: Type `/exit` to close the E2EE tunnel and quit the terminal UI.

---

## 🌐 Self-Hosting the Relay Server

The relay server is a lightweight binary residing in [render-server/](/render-server). You can easily deploy it on [Render.com](https://render.com) or any Linux VPS.

### Local Run
```bash
cd render-server
cargo run
```
*The server starts listening on port `10000` (configurable via the `PORT` environment variable).*

### Deploy to Render
We provide a [render.yaml](/render-server/render.yaml) file for one-click setup:
1.  Fork this repository.
2.  Go to Render, create a new **Blueprint**, and connect your repository.
3.  Deploy! It will automatically build and start the server.

To point your client to your custom server, set the `RELAY_SERVER` environment variable before launching:
```bash
# PowerShell
$env:RELAY_SERVER="your-custom-relay-app.onrender.com"
cargo run --release stage
```

---

## 🔒 Security Posture

*   **Forward Secrecy**: Keys are ephemeral and regenerated for every single session. Once you exit `/exit`, the keys are zeroized in memory, making past session captures impossible to decrypt.
*   **Zero-Knowledge Relay**: Since the relay server only stitches connections, it never sees the decrypted payload. It operates on a strict zero-knowledge level.
*   **Memory Security**: Cryptographic key bytes are zeroized (`zeroize` crate) when the `SecureSession` drops to prevent key extraction from core dumps.

## 📄 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

