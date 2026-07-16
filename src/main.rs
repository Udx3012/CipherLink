mod crypto;
mod network;
mod ui;

use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;
use colored::*;
use rand::Rng;
use network::RawConnection;
use ui::{AppEvent, AppState};

const DEFAULT_RELAY: &str = "fsociety-qzjr.onrender.com";

fn show_boot_animation() {
    let logo = r#"
XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
XX                                                                          XX
XX   MMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMM   XX
XX   MMMMMMMMMMMMMMMMMMMMMssssssssssssssssssssssssssMMMMMMMMMMMMMMMMMMMMM   XX
XX   MMMMMMMMMMMMMMMMss'''                          '''ssMMMMMMMMMMMMMMMM   XX
XX   MMMMMMMMMMMMyy''                                    ''yyMMMMMMMMMMMM   XX
XX   MMMMMMMMyy''                                            ''yyMMMMMMMM   XX
XX   MMMMMy''                                                    ''yMMMMM   XX
XX   MMMy'                                                          'yMMM   XX
XX   Mh'                                                              'hM   XX
XX   -                                                                  -   XX
XX                                                                          XX
XX   ::                                                                ::   XX
XX   MMhh.        ..hhhhhh..                      ..hhhhhh..        .hhMM   XX
XX   MMMMMh   ..hhMMMMMMMMMMhh.                .hhMMMMMMMMMMhh..   hMMMMM   XX
XX   ---MMM .hMMMMdd:::dMMMMMMMhh..        ..hhMMMMMMMd:::ddMMMMh. MMM---   XX
XX   MMMMMM MMmm''      'mmMMMMMMMMyy.  .yyMMMMMMMMmm'      ''mmMM MMMMMM   XX
XX   ---mMM ''             'mmMMMMMMMM  MMMMMMMMmm'             '' MMm---   XX
XX   yyyym'    .              'mMMMMm'  'mMMMMm'              .    'myyyy   XX
XX   mm''    .y'     ..yyyyy..  ''''      ''''  ..yyyyy..     'y.    ''mm   XX
XX           MN    .sMMMMMMMMMss.   .    .   .ssMMMMMMMMMs.    NM           XX
XX           N`    MMMMMMMMMMMMMN   M    M   NMMMMMMMMMMMMM    `N           XX
XX            +  .sMNNNNNMMMMMN+   `N    N`   +NMMMMMNNNNNMs.  +            XX
XX              o+++     ++++Mo    M      M    oM++++     +++o              XX
XX                                oo      oo                                XX
XX           oM                 oo          oo                 Mo           XX
XX         oMMo                M              M                oMMo         XX
XX       +MMMM                 s              s                 MMMM+       XX
XX      +MMMMM+            +++NNNN+        +NNNN+++            +MMMMM+      XX
XX     +MMMMMMM+       ++NNMMMMMMMMN+    +NMMMMMMMMNN++       +MMMMMMM+     XX
XX     MMMMMMMMMNN+++NNMMMMMMMMMMMMMMNNNNMMMMMMMMMMMMMMNN+++NNMMMMMMMMM     XX
XX     yMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMy     XX
XX   m  yMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMy  m   XX
XX   MMm yMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMy mMM   XX
XX   MMMm .yyMMMMMMMMMMMMMMMM     MMMMMMMMMM     MMMMMMMMMMMMMMMMyy. mMMM   XX
XX   MMMMd   ''''hhhhh       odddo          obbbo        hhhh''''   dMMMM   XX
XX   MMMMMd             'hMMMMMMMMMMddddddMMMMMMMMMMh'             dMMMMM   XX
XX   MMMMMMd              'hMMMMMMMMMMMMMMMMMMMMMMh'              dMMMMMM   XX
XX   MMMMMMM-               ''ddMMMMMMMMMMMMMMdd''               -MMMMMMM   XX
XX   MMMMMMMM                   '::dddddddd::'                   MMMMMMMM   XX
XX   MMMMMMMM-                                                  -MMMMMMMM   XX
XX   MMMMMMMMM                                                  MMMMMMMMM   XX
XX   MMMMMMMMMy                                                yMMMMMMMMM   XX
XX   MMMMMMMMMMy.                                            .yMMMMMMMMMM   XX
XX   MMMMMMMMMMMMy.                                        .yMMMMMMMMMMMM   XX
XX   MMMMMMMMMMMMMMy.                                    .yMMMMMMMMMMMMMM   XX
XX   MMMMMMMMMMMMMMMMs.                                .sMMMMMMMMMMMMMMMM   XX
XX   MMMMMMMMMMMMMMMMMMss.           ....           .ssMMMMMMMMMMMMMMMMMM   XX
XX   MMMMMMMMMMMMMMMMMMMMNo         oNNNNo         oNMMMMMMMMMMMMMMMMMMMM   XX
XX                                                                          XX
XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

    .o88o.                               o8o                .
    888 `"                               `"'              .o8
   o888oo   .oooo.o  .ooooo.   .ooooo.  oooo   .ooooo.  .o888oo oooo    ooo
    888    d88(  "8 d88' `88b d88' `"Y8 `888  d88' `88b   888    `88.  .8'
    888    `"Y88b.  888   888 888        888  888ooo888   888     `88..8'
    888    o.  )88b 888   888 888   .o8  888  888    .o   888 .    `888'
   o888o   8""888P' `Y8bod8P' `Y8bod8P' o888o `Y8bod8P'   "888"      d8'
                                                                .o...P'
                                                                `UDXY'
    "#;

    println!("{}", logo.bright_green());
    print!("{}", " [fsociety] INITIALIZING PEER-TO-PEER PROTOCOL...".green());
    let _ = std::io::Write::flush(&mut std::io::stdout());
    thread::sleep(Duration::from_millis(400));
    
    let stages = [
        " [fsociety] LOADING ENCRYPTION MODULE (Curve25519)... DONE",
        " [fsociety] CONFIGURING AUTHENTICATED ENCRYPTION (ChaCha20-Poly1305)... DONE",
        " [fsociety] ENABLING SHADOW MEMORY WIPER (Zeroize)... ACTIVE",
        " [fsociety] ESTABLISHING SECURE TUNNEL...",
    ];

    for stage in &stages {
        println!();
        print!("{}", stage.green());
        let _ = std::io::Write::flush(&mut std::io::stdout());
        thread::sleep(Duration::from_millis(300));
    }
    println!("\n");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("{}", "FSOCIETY v2.0 (E2EE chat/transfer)".bright_green());
        println!("Usage:");
        println!("  fsociety stage [code]     - Create a secure room globally (optional custom 6-digit code)");
        println!("  fsociety stage --local    - Listen for local connection on port 8766");
        println!("  fsociety connect <code>   - Connect to a global room");
        println!("  fsociety connect --local <ip:port> - Connect directly to a local peer");
    }

    let username_file = ".fsociety_user";
    let our_username = if let Ok(mut file) = std::fs::File::open(username_file) {
        let mut contents = String::new();
        use std::io::Read;
        if file.read_to_string(&mut contents).is_ok() && !contents.trim().is_empty() {
            contents.trim().to_string()
        } else {
            prompt_username(username_file)?
        }
    } else {
        prompt_username(username_file)?
    };

    show_boot_animation();

    let (conn, room_code, conn_status) = match args[1].as_str() {
        "stage" => {
            if args.len() >= 3 && args[2] == "--local" {
                println!("{}", " [fsociety] Waiting for direct TCP connection on port 8766...".green());
                let conn = network::listen_tcp(8766)?;
                (conn, "LOCAL".to_string(), "DIRECT (TCP)".to_string())
            } else {
                let relay_host = std::env::var("RELAY_SERVER")
                    .unwrap_or_else(|_| DEFAULT_RELAY.to_string());
                let room = if args.len() >= 3 {
                    let custom_code = &args[2];
                    if custom_code.len() == 6 && custom_code.chars().all(|c| c.is_numeric()) {
                        custom_code.clone()
                    } else {
                        println!("{}", format!(" [fsociety] Warning: '{}' is not a valid 6-digit numeric code. Generating random code.", custom_code).yellow());
                        let code: u32 = rand::thread_rng().gen_range(100000..999999);
                        format!("{:06}", code)
                    }
                } else {
                    let code: u32 = rand::thread_rng().gen_range(100000..999999);
                    format!("{:06}", code)
                };
                let wss_url = format!("ws://{}/ws/{}", relay_host, room);
                println!("{}", format!(" [fsociety] Registering room code: {}", room).bright_green());
                println!("{}", format!(" [fsociety] Target address: ws://{}/ws/{}", relay_host, room).green());
                
                // Print QR code for mobile web interface fallback
                let _ = qr2term::print_qr(&format!("http://{}/room/{}", relay_host, room));

                let conn = network::connect_ws(&wss_url)?;
                (conn, room, "BLIND RELAY (WS)".to_string())
            }
        }
        "connect" => {
            if args.len() < 3 {
                println!("Error: Specify room code or address");
                return Ok(());
            }
            if args[2] == "--local" {
                if args.len() < 4 {
                    println!("Error: Specify local <ip:port> address");
                    return Ok(());
                }
                let addr = &args[3];
                println!("{}", format!(" [fsociety] Connecting directly to {}...", addr).green());
                let conn = network::connect_tcp(addr)?;
                (conn, "LOCAL".to_string(), "DIRECT (TCP)".to_string())
            } else {
                let room = &args[2];
                let relay_host = std::env::var("RELAY_SERVER")
                    .unwrap_or_else(|_| DEFAULT_RELAY.to_string());
                let wss_url = format!("ws://{}/ws/{}", relay_host, room);
                println!("{}", format!(" [fsociety] Dialing global room {}...", room).green());
                let conn = network::connect_ws(&wss_url)?;
                (conn, room.clone(), "BLIND RELAY (WS)".to_string())
            }
        }
        _ => {
            println!("Unknown command");
            return Ok(());
        }
    };

    println!("{}", " [fsociety] Tunnel established. Initiating E2EE handshake...".bright_green());
    let (conn, session) = network::perform_handshake(conn)?;
    let session = Arc::new(session);
    let fingerprint = session.fingerprint().to_string();
    println!("{}", format!(" [fsociety] SECURE SESSION ACTIVE. Fingerprint: {}", fingerprint).bright_green());
    thread::sleep(Duration::from_secs(1));

    // Channels for Event communication
    let (event_tx, event_rx) = mpsc::channel::<AppEvent>();
    let (cmd_tx, cmd_rx) = mpsc::channel::<String>();

    // Duplicate connections for read/write handling
    // Since we need to read & write from separate threads:
    // We can wrap RawConnection in an Arc<Mutex<>> or duplicate it.
    // Wrap RawConnection in Arc<Mutex<>> so it can be shared safely.
    let shared_conn = Arc::new(std::sync::Mutex::new(conn));

    // Spawn network receive thread
    let conn_rx = Arc::clone(&shared_conn);
    let session_rx = Arc::clone(&session);
    let event_tx_rx = event_tx.clone();
    thread::spawn(move || {
        network_receive_loop(conn_rx, session_rx, event_tx_rx);
    });

    // Spawn network send thread
    let conn_tx = Arc::clone(&shared_conn);
    let session_tx = Arc::clone(&session);
    let event_tx_tx = event_tx.clone();
    let our_username_tx = our_username.clone();
    thread::spawn(move || {
        network_send_loop(conn_tx, session_tx, cmd_rx, event_tx_tx, our_username_tx);
    });

    // Run the UI
    let mut state = AppState::new(room_code, fingerprint, conn_status, our_username.clone());
    ui::start_ui_loop(&mut state, event_rx, cmd_tx)?;

    Ok(())
}

fn network_receive_loop(
    conn: Arc<std::sync::Mutex<RawConnection>>,
    session: Arc<crypto::SecureSession>,
    event_tx: mpsc::Sender<AppEvent>,
) {
    let mut current_file: Option<(std::fs::File, String, u64, u64)> = None;
    
    // Set non-blocking mode
    if let Ok(guard) = conn.lock() {
        let _ = guard.set_nonblocking(true);
    }

    loop {
        let packet = {
            if let Ok(mut guard) = conn.lock() {
                guard.read_packet()
            } else {
                Err("Lock failed")
            }
        };

        match packet {
            Ok(Some(encrypted_payload)) => {
                match session.decrypt(&encrypted_payload) {
                    Ok(plaintext) => {
                        if plaintext.is_empty() { continue; }
                        let msg_type = plaintext[0];
                        match msg_type {
                            0x01 => {
                                if let Ok(text) = String::from_utf8(plaintext[1..].to_vec()) {
                                    let _ = event_tx.send(AppEvent::MessageReceived("Peer".to_string(), text));
                                }
                            }
                            0x06 => {
                                if let Ok(name) = String::from_utf8(plaintext[1..].to_vec()) {
                                    let _ = event_tx.send(AppEvent::PeerUsername(name));
                                }
                            }
                            0x02 => {
                                if plaintext.len() >= 9 {
                                    let mut size_bytes = [0u8; 8];
                                    size_bytes.copy_from_slice(&plaintext[1..9]);
                                    let file_size = u64::from_be_bytes(size_bytes);
                                    if let Ok(filename) = String::from_utf8(plaintext[9..].to_vec()) {
                                        let save_path = format!("downloaded_{}", filename);
                                        if let Ok(file) = std::fs::OpenOptions::new()
                                            .create(true)
                                            .write(true)
                                            .truncate(true)
                                            .open(&save_path)
                                        {
                                            current_file = Some((file, filename.clone(), file_size, 0));
                                            let _ = event_tx.send(AppEvent::SystemMessage(format!("[FILE] Downloading: {} ({} bytes)", filename, file_size)));
                                        }
                                    }
                                }
                            }
                            0x03 => {
                                if plaintext.len() >= 5 {
                                    if let Some((ref mut file, ref filename, total_size, ref mut bytes_written)) = current_file {
                                        let chunk_data = &plaintext[5..];
                                        use std::io::Write;
                                        if file.write_all(chunk_data).is_ok() {
                                            *bytes_written += chunk_data.len() as u64;
                                            let progress = if total_size > 0 { *bytes_written as f64 / total_size as f64 } else { 1.0 };
                                            let _ = event_tx.send(AppEvent::FileProgress(filename.clone(), progress));
                                        }
                                    }
                                }
                            }
                            0x04 => {
                                if let Some((_, filename, _, _)) = current_file.take() {
                                    let _ = event_tx.send(AppEvent::FileComplete(filename));
                                }
                            }
                            0x05 => {
                                let _ = event_tx.send(AppEvent::ConnectionClosed);
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(_) => {
                        let _ = event_tx.send(AppEvent::SystemMessage("[WARN] Failed to decrypt packet".to_string()));
                    }
                }
            }
            Ok(None) => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(_) => {
                let _ = event_tx.send(AppEvent::ConnectionClosed);
                break;
            }
        }
    }
}

fn network_send_loop(
    conn: Arc<std::sync::Mutex<RawConnection>>,
    session: Arc<crypto::SecureSession>,
    cmd_rx: mpsc::Receiver<String>,
    event_tx: mpsc::Sender<AppEvent>,
    our_username: String,
) {
    // Send username initially
    let mut init_payload = vec![0x06];
    init_payload.extend_from_slice(our_username.as_bytes());
    if let Ok(ciphertext) = session.encrypt(&init_payload) {
        if let Ok(mut guard) = conn.lock() {
            let _ = guard.write_packet(&ciphertext);
        }
    }

    loop {
        match cmd_rx.recv() {
            Ok(cmd) => {
                if cmd.starts_with("/send ") {
                    let filepath = cmd.strip_prefix("/send ").unwrap().trim();
                    let path = std::path::Path::new(filepath);
                    if !path.exists() {
                        let _ = event_tx.send(AppEvent::SystemMessage(format!("[ERROR] File not found: {}", filepath)));
                        continue;
                    }
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    let file_size = match path.metadata() {
                        Ok(meta) => meta.len(),
                        Err(_) => {
                            let _ = event_tx.send(AppEvent::SystemMessage("[ERROR] Failed to read metadata".to_string()));
                            continue;
                        }
                    };

                    let _ = event_tx.send(AppEvent::SystemMessage(format!("[FILE] Outgoing file: {}", filename)));

                    // 1. File Start Packet
                    let mut start_payload = vec![0x02];
                    start_payload.extend_from_slice(&file_size.to_be_bytes());
                    start_payload.extend_from_slice(filename.as_bytes());
                    if let Ok(ciphertext) = session.encrypt(&start_payload) {
                        if let Ok(mut guard) = conn.lock() {
                            if guard.write_packet(&ciphertext).is_err() { break; }
                        }
                    }

                    // 2. Chunks Stream
                    if let Ok(mut file) = std::fs::File::open(filepath) {
                        let mut buffer = vec![0u8; 64 * 1024];
                        let mut chunk_idx = 0u32;
                        let mut bytes_sent = 0u64;
                        let mut success = true;

                        loop {
                            use std::io::Read;
                            match file.read(&mut buffer) {
                                Ok(0) => break,
                                Ok(n) => {
                                    let mut chunk_payload = vec![0x03];
                                    chunk_payload.extend_from_slice(&chunk_idx.to_be_bytes());
                                    chunk_payload.extend_from_slice(&buffer[..n]);
                                    if let Ok(ciphertext) = session.encrypt(&chunk_payload) {
                                        if let Ok(mut guard) = conn.lock() {
                                            if guard.write_packet(&ciphertext).is_err() {
                                                success = false;
                                                break;
                                            }
                                        } else {
                                            success = false;
                                            break;
                                        }
                                    } else {
                                        success = false;
                                        break;
                                    }
                                    chunk_idx += 1;
                                    bytes_sent += n as u64;
                                    let progress = if file_size > 0 { bytes_sent as f64 / file_size as f64 } else { 1.0 };
                                    let _ = event_tx.send(AppEvent::FileProgress(filename.clone(), progress));
                                }
                                Err(_) => {
                                    success = false;
                                    break;
                                }
                            }
                            thread::sleep(Duration::from_millis(5));
                        }

                        if success {
                            let complete_payload = vec![0x04];
                            if let Ok(ciphertext) = session.encrypt(&complete_payload) {
                                if let Ok(mut guard) = conn.lock() {
                                    let _ = guard.write_packet(&ciphertext);
                                }
                            }
                            let _ = event_tx.send(AppEvent::SystemMessage(format!("[SUCCESS] Finished sending: {}", filename)));
                        } else {
                            let _ = event_tx.send(AppEvent::SystemMessage(format!("[ERROR] Transfer failed: {}", filename)));
                        }
                    }
                } else if cmd == "/exit" {
                    let exit_payload = vec![0x05];
                    if let Ok(ciphertext) = session.encrypt(&exit_payload) {
                        if let Ok(mut guard) = conn.lock() {
                            let _ = guard.write_packet(&ciphertext);
                        }
                    }
                    break;
                } else {
                    let mut msg_payload = vec![0x01];
                    msg_payload.extend_from_slice(cmd.as_bytes());
                    if let Ok(ciphertext) = session.encrypt(&msg_payload) {
                        let mut failed = false;
                        if let Ok(mut guard) = conn.lock() {
                            if guard.write_packet(&ciphertext).is_err() { failed = true; }
                        } else { failed = true; }
                        
                        if !failed {
                            let _ = event_tx.send(AppEvent::Input(cmd));
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }
}

fn prompt_username(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Write;
    print!("Enter your username (this will be saved): ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let username = input.trim().to_string();
    let username = if username.is_empty() {
        "Anonymous".to_string()
    } else {
        username
    };
    if let Ok(mut file) = std::fs::File::create(filename) {
        let _ = file.write_all(username.as_bytes());
    }
    Ok(username)
}