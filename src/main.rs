mod request_detail;
mod request_overview;
mod response_config;

use chrono::Local;
use eframe::egui;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming};
use hyper_util::rt::TokioIo;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::TcpListener;

#[derive(Clone, Debug)]
pub struct HttpRequest {
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub query_params: Vec<(String, String)>,
    pub headers: Vec<(String, String)>,
    pub remote_addr: String,
    pub body: String,
    pub body_size: usize,
}

struct HttpServerApp {
    requests: Arc<Mutex<Vec<HttpRequest>>>,
    port: u16,
    port_input: String,
    selected_request: Option<usize>,
    port_change_tx: Sender<u16>,
    server_status: Arc<Mutex<String>>,
    last_working_port: Arc<Mutex<u16>>,
    error_message: Option<String>,
    error_timestamp: Option<Instant>,
    response_config: Arc<Mutex<response_config::ResponseConfig>>,
    active_tab: AppTab,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppTab {
    RequestDetails,
    ResponseConfig,
}

impl HttpServerApp {
    fn new(
        port: u16,
        port_change_tx: Sender<u16>,
        server_status: Arc<Mutex<String>>,
        last_working_port: Arc<Mutex<u16>>,
    ) -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            port,
            port_input: port.to_string(),
            selected_request: None,
            port_change_tx,
            server_status,
            last_working_port,
            error_message: None,
            error_timestamp: None,
            response_config: Arc::new(Mutex::new(response_config::ResponseConfig::default())),
            active_tab: AppTab::RequestDetails,
        }
    }
}

impl eframe::App for HttpServerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        // Check if error message should be cleared (after 5 seconds)
        if let Some(timestamp) = self.error_timestamp
            && timestamp.elapsed() > Duration::from_secs(5)
        {
            self.error_message = None;
            self.error_timestamp = None;
        }

        // Check if server is in error state and reset port input to last working port
        {
            let status = self.server_status.lock().unwrap();
            if status.contains("Error") {
                // Capture error message if not already set
                if self.error_message.is_none() {
                    self.error_message = Some(status.clone());
                    self.error_timestamp = Some(Instant::now());
                }

                let last_working = *self.last_working_port.lock().unwrap();
                if self.port != last_working {
                    self.port = last_working;
                    self.port_input = last_working.to_string();
                    // Tell server to rebind to the last working port
                    let _ = self.port_change_tx.send(last_working);
                }
            }
        }

        // Top panel with title and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.heading("Inspector HTTP");
                ui.separator();

                // Port configuration
                ui.label("Port:");
                let port_edit = egui::TextEdit::singleline(&mut self.port_input)
                    .desired_width(60.0)
                    .hint_text("8080");

                if ui.add(port_edit).lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(new_port) = self.port_input.parse::<u16>() {
                        if new_port != self.port && new_port > 0 {
                            let _ = self.port_change_tx.send(new_port);
                            self.port = new_port;
                        } else if new_port == 0 {
                            // Port 0 is invalid, reset to last working port
                            let last_working = *self.last_working_port.lock().unwrap();
                            self.port_input = last_working.to_string();
                        }
                    } else {
                        // Invalid number, reset to last working port
                        let last_working = *self.last_working_port.lock().unwrap();
                        self.port_input = last_working.to_string();
                    }
                }

                ui.separator();
                let requests = self.requests.lock().unwrap();
                ui.label(format!("Total Requests: {}", requests.len()));
            });
            ui.add_space(5.0);
        });

        // Bottom status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                // Show error message if present, otherwise show server status
                if let Some(ref error_msg) = self.error_message {
                    ui.label(
                        egui::RichText::new("●")
                            .color(egui::Color32::RED)
                            .size(16.0),
                    );
                    ui.label(egui::RichText::new(error_msg).color(egui::Color32::RED));
                } else {
                    let status = self.server_status.lock().unwrap();
                    let status_color = if status.contains("Listening") {
                        egui::Color32::GREEN
                    } else if status.contains("Error") {
                        egui::Color32::RED
                    } else {
                        egui::Color32::YELLOW
                    };

                    ui.label(egui::RichText::new("●").color(status_color).size(16.0));
                    ui.label(egui::RichText::new(&*status).color(status_color));
                }
            });
            ui.add_space(3.0);
        });

        // Left panel - Request list overview
        egui::SidePanel::left("request_list")
            .default_width(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                let requests = self.requests.lock().unwrap();
                request_overview::render_request_overview(
                    ui,
                    &requests,
                    &mut self.selected_request,
                );
            });

        // Right panel - Tabbed view (Request Details / Response Config)
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.active_tab,
                    AppTab::RequestDetails,
                    "📥 Request Details",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    AppTab::ResponseConfig,
                    "📤 Response Config",
                );
            });
            ui.separator();

            // Tab content
            match self.active_tab {
                AppTab::RequestDetails => {
                    let requests = self.requests.lock().unwrap();
                    let selected_request = self.selected_request.and_then(|idx| requests.get(idx));
                    request_detail::render_request_detail(ui, selected_request);
                }
                AppTab::ResponseConfig => {
                    let mut config = self.response_config.lock().unwrap();
                    response_config::render_response_config(ui, &mut config);
                }
            }
        });
    }
}

async fn handle_request(
    req: Request<Incoming>,
    remote_addr: String,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
    response_config: Arc<Mutex<response_config::ResponseConfig>>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let method = req.method().to_string();

    // Capture full URI including query parameters
    let path = if let Some(query) = req.uri().query() {
        format!("{}?{}", req.uri().path(), query)
    } else {
        req.uri().path().to_string()
    };

    // Parse query parameters
    let query_params: Vec<(String, String)> = req
        .uri()
        .query()
        .map(|q| {
            q.split('&')
                .filter_map(|pair| {
                    let mut split = pair.splitn(2, '=');
                    match (split.next(), split.next()) {
                        (Some(key), Some(value)) => Some((
                            urlencoding::decode(key).unwrap_or_default().to_string(),
                            urlencoding::decode(value).unwrap_or_default().to_string(),
                        )),
                        (Some(key), None) => Some((
                            urlencoding::decode(key).unwrap_or_default().to_string(),
                            String::new(),
                        )),
                        _ => None,
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    let headers: Vec<(String, String)> = req
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or("<binary>").to_string(),
            )
        })
        .collect();

    // Read the request body
    let body_bytes = req.collect().await?.to_bytes();
    let body_size = body_bytes.len();
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    let http_req = HttpRequest {
        timestamp,
        method,
        path,
        query_params,
        headers,
        remote_addr,
        body,
        body_size,
    };

    requests.lock().unwrap().push(http_req);

    // Build response using configured status code and body
    let config = response_config.lock().unwrap();
    let response_body = config.response_body.clone();
    let status_code = config.status_code;
    drop(config); // Release lock early

    // Build the response - this shouldn't fail with valid status codes
    let response = Response::builder()
        .status(status_code)
        .body(Full::new(Bytes::from(response_body)))
        .unwrap_or_else(|e| {
            eprintln!("Error building response: {}", e);
            // Fallback to a simple 200 OK response
            Response::new(Full::new(Bytes::from("OK\n")))
        });

    Ok(response)
}

fn find_available_port(start_port: u16) -> u16 {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        for port in start_port..=65535 {
            let addr = format!("0.0.0.0:{}", port);
            if TcpListener::bind(&addr).await.is_ok() {
                return port;
            }
        }
        // Fallback to start_port if no port is available (unlikely)
        start_port
    })
}

fn main() {
    // Find first available port starting from 8080
    let available_port = find_available_port(8080);

    let (port_tx, port_rx): (Sender<u16>, Receiver<u16>) = channel();
    let port_rx = Arc::new(Mutex::new(port_rx));
    let server_status = Arc::new(Mutex::new(String::from("Starting...")));
    let last_working_port = Arc::new(Mutex::new(available_port));

    let app = HttpServerApp::new(
        available_port,
        port_tx,
        Arc::clone(&server_status),
        Arc::clone(&last_working_port),
    );
    let requests = Arc::clone(&app.requests);
    let response_config = Arc::clone(&app.response_config);
    let initial_port = app.port;

    // Spawn server thread that can restart on port changes
    let port_rx_clone = Arc::clone(&port_rx);
    let last_working_clone = Arc::clone(&last_working_port);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut current_port = initial_port;

        loop {
            let requests_clone = Arc::clone(&requests);
            let response_config_clone = Arc::clone(&response_config);
            let status_clone = Arc::clone(&server_status);
            let port_rx_clone2 = Arc::clone(&port_rx_clone);
            let last_working_clone2 = Arc::clone(&last_working_clone);

            // Update status
            *status_clone.lock().unwrap() = format!("Listening on http://0.0.0.0:{}", current_port);

            // Run server with cancellation support
            rt.block_on(async {
                match run_server_cancellable(
                    current_port,
                    requests_clone,
                    response_config_clone,
                    port_rx_clone2,
                    status_clone,
                    last_working_clone2,
                )
                .await
                {
                    Ok(new_port) => {
                        current_port = new_port;
                        println!("Restarting server on port {}", current_port);
                    }
                    Err(e) => {
                        eprintln!("Server error: {}", e);
                        let error_msg = format!("Error: {}", e);
                        *server_status.lock().unwrap() = error_msg;

                        // Wait for user to provide a new port instead of retrying same port
                        if let Ok(new_port) = port_rx_clone.lock().unwrap().recv() {
                            current_port = new_port;
                            println!("New port received after error: {}", current_port);
                        }
                    }
                }
            });
        }
    });

    // Load application icon
    let icon_data = include_bytes!("../assets/icon-256.png");
    let icon_image = image::load_from_memory(icon_data)
        .expect("Failed to load icon")
        .to_rgba8();
    let (icon_width, icon_height) = icon_image.dimensions();
    let icon = egui::IconData {
        rgba: icon_image.into_raw(),
        width: icon_width,
        height: icon_height,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("InspectorHTTP")
            .with_inner_size([800.0, 600.0])
            .with_title("Inspector HTTP")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native("Inspector HTTP", options, Box::new(|_cc| Ok(Box::new(app)))).unwrap();
}

async fn run_server_cancellable(
    port: u16,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
    response_config: Arc<Mutex<response_config::ResponseConfig>>,
    port_rx: Arc<Mutex<Receiver<u16>>>,
    _server_status: Arc<Mutex<String>>,
    last_working_port: Arc<Mutex<u16>>,
) -> Result<u16, String> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            return Err(format!("Failed to bind to {}: {}", addr, e));
        }
    };

    // Successfully bound - update last working port
    *last_working_port.lock().unwrap() = port;
    println!("HTTP Server listening on {}", addr);

    loop {
        // Check for port change request (non-blocking)
        if let Ok(new_port) = port_rx.lock().unwrap().try_recv() {
            println!("Port change requested: {} -> {}", port, new_port);
            return Ok(new_port);
        }

        // Accept connections with timeout
        let accept_result =
            tokio::time::timeout(std::time::Duration::from_millis(100), listener.accept()).await;

        match accept_result {
            Ok(Ok((stream, remote_addr))) => {
                let io = TokioIo::new(stream);
                let requests = Arc::clone(&requests);
                let response_config = Arc::clone(&response_config);
                let remote_addr_str = remote_addr.to_string();

                tokio::task::spawn(async move {
                    let service = service_fn(move |req| {
                        handle_request(
                            req,
                            remote_addr_str.clone(),
                            Arc::clone(&requests),
                            Arc::clone(&response_config),
                        )
                    });

                    if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
            Ok(Err(e)) => {
                eprintln!("Error accepting connection: {}", e);
            }
            Err(_) => {
                // Timeout - continue loop to check for port changes
            }
        }
    }
}
