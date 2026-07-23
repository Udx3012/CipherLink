use std::io;
use std::time::Duration;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap, Padding},
    Terminal,
};

pub enum AppEvent {
    Input(String),
    MessageReceived(String, String), // sender, message
    SystemMessage(String),
    FileProgress(String, f64), // filename, progress (0.0..=1.0)
    FileComplete(String),
    ConnectionClosed,
    PeerUsername(String),
}

pub struct AppState {
    pub messages: Vec<(String, String)>,
    pub system_logs: Vec<String>,
    pub input: String,
    pub room_code: String,
    pub fingerprint: String,
    pub conn_status: String,
    pub transfer_filename: Option<String>,
    pub transfer_progress: f64,
    pub our_username: String,
    pub peer_username: String,
}

impl AppState {
    pub fn new(room_code: String, fingerprint: String, conn_status: String, our_username: String) -> Self {
        Self {
            messages: Vec::new(),
            system_logs: vec!["[INFO] Ephemeral tunnel established.".to_string()],
            input: String::new(),
            room_code,
            fingerprint,
            conn_status,
            transfer_filename: None,
            transfer_progress: 0.0,
            our_username,
            peer_username: "Peer".to_string(),
        }
    }
}

pub fn start_ui_loop(
    state: &mut AppState,
    rx: Receiver<AppEvent>,
    tx: Sender<String>, // sends commands/messages back to main thread
) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let cursor_visible = true;
    let mut ticks = 0;

    loop {
        // Redraw terminal
        terminal.draw(|f| {
            let size = f.size();
            
            // Check terminal minimum size
            if size.width < 50 || size.height < 15 {
                let warning = Paragraph::new("Terminal too small! Increase window size.").style(Style::default().fg(Color::Red));
                f.render_widget(warning, size);
                return;
            }

            // Main vertical layout (Chat/Sidebar on top, Input at bottom)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(size);

            // Horizontal split (Chat left 70%, Sidebar right 30%)
            let top_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(chunks[0]);

            // Widgets
            // 1. Chat Window
            let mut chat_content = String::new();
            for (sender, msg) in &state.messages {
                chat_content.push_str(&format!("[{}] {}\n", sender, msg));
            }
            if chat_content.is_empty() {
                chat_content = "\n\n  --- No messages in this session ---".to_string();
            }
            
            let chat_paragraph = Paragraph::new(chat_content)
                .block(Block::default()
                    .title(" 🔐 CIPHERLINK [E2EE CHAT CHANNEL] ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .padding(Padding::new(1, 1, 1, 1)))
                .style(Style::default().fg(Color::LightGreen))
                .wrap(Wrap { trim: false });
            f.render_widget(chat_paragraph, top_chunks[0]);

            // 2. Sidebar
            let mut sidebar_content = format!(
                "⚡ CIPHERLINK v2.0\n\n📶 Connection Status\n  {}\n\n🗝️ Room Access Code\n  {}\n\n🔒 Fingerprint\n  {}\n\n",
                state.conn_status, state.room_code, state.fingerprint
            );

            if let Some(ref filename) = state.transfer_filename {
                sidebar_content.push_str(&format!(
                    "📁 Transferring File:\n  {}\n  Progress: {:.1}%\n  [{}{}]\n\n",
                    filename,
                    state.transfer_progress * 100.0,
                    "#".repeat((state.transfer_progress * 10.0) as usize),
                    ".".repeat(10 - (state.transfer_progress * 10.0) as usize)
                ));
            }

            sidebar_content.push_str("⚡ System Diagnostics\n");
            for log in state.system_logs.iter().rev().take(5) {
                sidebar_content.push_str(&format!("  {}\n", log));
            }

            let sidebar_paragraph = Paragraph::new(sidebar_content)
                .block(Block::default()
                    .title(" ⚙️ SYSTEM STATE ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .padding(Padding::new(1, 1, 1, 1)))
                .style(Style::default().fg(Color::Green))
                .wrap(Wrap { trim: true });
            f.render_widget(sidebar_paragraph, top_chunks[1]);

            // 3. Input Bar
            let input_prompt = if cursor_visible && (ticks % 2 == 0) {
                format!("{}_", state.input)
            } else {
                format!("{} ", state.input)
            };

            let input_paragraph = Paragraph::new(input_prompt)
                .block(Block::default()
                    .title(" 💬 PAYLOAD INPUT ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green)))
                .style(Style::default().fg(Color::LightGreen));
            f.render_widget(input_paragraph, chunks[1]);
        })?;

        // Handle events from background network thread or keystrokes
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            if !state.input.trim().is_empty() {
                                let cmd = state.input.clone();
                                state.input.clear();
                                // Send to network / orchestrator
                                let _ = tx.send(cmd);
                            }
                        }
                        KeyCode::Char(c) => {
                            state.input.push(c);
                        }
                        KeyCode::Backspace => {
                            state.input.pop();
                        }
                        KeyCode::Esc => {
                            let _ = tx.send("/exit".to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        // Process network events
        while let Ok(net_event) = rx.try_recv() {
            match net_event {
                AppEvent::PeerUsername(name) => {
                    state.peer_username = name.clone();
                    state.system_logs.push(format!("[INFO] Peer identified as: {}", name));
                }
                AppEvent::Input(text) => {
                    state.messages.push((state.our_username.clone(), text));
                }
                AppEvent::MessageReceived(sender, text) => {
                    let display_sender = if sender == "Peer" { state.peer_username.clone() } else { sender };
                    state.messages.push((display_sender, text));
                }
                AppEvent::SystemMessage(text) => {
                    state.system_logs.push(text);
                }
                AppEvent::FileProgress(filename, progress) => {
                    state.transfer_filename = Some(filename);
                    state.transfer_progress = progress;
                }
                AppEvent::FileComplete(filename) => {
                    state.system_logs.push(format!("[SUCCESS] File saved: {}", filename));
                    state.transfer_filename = None;
                    state.transfer_progress = 0.0;
                }
                AppEvent::ConnectionClosed => {
                    state.system_logs.push("[ERROR] Peer disconnected.".to_string());
                    state.conn_status = "DISCONNECTED".to_string();
                }
            }
        }

        ticks += 1;
        if ticks > 100 { ticks = 0; }

        // Exit if connection status changed to closed and the exit command was processed
        if state.conn_status == "DISCONNECTED" {
            thread::sleep(Duration::from_secs(2));
            break;
        }
        if state.conn_status == "CLOSED_BY_USER" {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
