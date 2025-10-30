use crate::HttpRequest;
use eframe::egui;
use egui_json_tree::JsonTree;

pub fn render_request_detail(ui: &mut egui::Ui, request: Option<&HttpRequest>) {
    if let Some(req) = request {
        ui.heading("Request Details");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Timestamp
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Timestamp:").strong());
                ui.label(egui::RichText::new(&req.timestamp).monospace());
            });
            ui.add_space(5.0);

            // Method and Path
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Method:").strong());
                ui.label(
                    egui::RichText::new(&req.method)
                        .strong()
                        .color(get_method_color(&req.method)),
                );
            });
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Path:").strong());
                ui.label(egui::RichText::new(&req.path).monospace());
            });
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("From:").strong());
                ui.label(egui::RichText::new(&req.remote_addr).monospace());
            });
            ui.add_space(10.0);

            // Query parameters section (if present)
            if !req.query_params.is_empty() {
                ui.separator();
                ui.label(
                    egui::RichText::new(format!("Query Parameters ({})", req.query_params.len()))
                        .heading(),
                );
                ui.add_space(5.0);

                render_query_params(ui, &req.query_params);

                ui.add_space(10.0);
            }

            // Headers section
            ui.separator();
            ui.label(egui::RichText::new(format!("Headers ({})", req.headers.len())).heading());
            ui.add_space(5.0);

            render_headers(ui, &req.headers);

            ui.add_space(10.0);

            // Body section
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Body ({} bytes)", req.body_size)).heading());

                if req.body_size > 0 {
                    ui.add_space(10.0);

                    // Copy to clipboard button
                    if ui.button("📋 Copy to Clipboard").clicked() {
                        ui.ctx().copy_text(req.body.clone());
                    }

                    // Save to file button
                    if ui.button("💾 Save to File").clicked() {
                        let body = req.body.clone();
                        let filename = generate_filename(&req.timestamp, &req.path, &req.headers);

                        std::thread::spawn(move || {
                            if let Some(path) =
                                rfd::FileDialog::new().set_file_name(&filename).save_file()
                            {
                                if let Err(e) = std::fs::write(&path, body) {
                                    eprintln!("Failed to save file: {}", e);
                                } else {
                                    println!("Body saved to: {:?}", path);
                                }
                            }
                        });
                    }
                }
            });
            ui.add_space(5.0);

            render_body(ui, &req.body, req.body_size, &req.headers);
        });
    } else {
        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new("Select a request from the list to view details").weak());
        });
    }
}

fn render_query_params(ui: &mut egui::Ui, query_params: &[(String, String)]) {
    egui::Grid::new("query_params_grid")
        .striped(true)
        .spacing([10.0, 5.0])
        .show(ui, |ui| {
            for (key, value) in query_params {
                ui.label(egui::RichText::new(key).strong());
                ui.label(egui::RichText::new(value).monospace());
                ui.end_row();
            }
        });
}

fn render_headers(ui: &mut egui::Ui, headers: &[(String, String)]) {
    egui::Grid::new("headers_grid")
        .striped(true)
        .spacing([10.0, 5.0])
        .show(ui, |ui| {
            for (name, value) in headers {
                ui.label(egui::RichText::new(name).strong());
                ui.label(egui::RichText::new(value).monospace());
                ui.end_row();
            }
        });
}

fn render_body(ui: &mut egui::Ui, body: &str, body_size: usize, headers: &[(String, String)]) {
    if body_size > 0 {
        // Check if content is JSON
        let content_type = headers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
            .map(|(_, value)| value.as_str())
            .unwrap_or("");

        let is_json = content_type.contains("application/json")
            || content_type.contains("text/json")
            || (body.trim_start().starts_with('{') || body.trim_start().starts_with('['));

        if is_json {
            // Try to parse and render as JSON tree
            match serde_json::from_str::<serde_json::Value>(body) {
                Ok(json_value) => {
                    // Determine default expansion based on size
                    // For large JSON (>100KB), start collapsed to avoid performance issues
                    let default_expand = if body_size > 100_000 {
                        egui_json_tree::DefaultExpand::None
                    } else {
                        egui_json_tree::DefaultExpand::All
                    };

                    egui::Frame::new()
                        .fill(egui::Color32::from_gray(30))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            // Show warning for very large JSON
                            if body_size > 100_000 {
                                ui.label(
                                    egui::RichText::new(
                                        format!("⚠ Large JSON ({:.1} KB) - expand nodes carefully for better performance",
                                        body_size as f32 / 1024.0)
                                    )
                                    .small()
                                    .color(egui::Color32::YELLOW)
                                );
                                ui.add_space(5.0);
                            }

                            egui::ScrollArea::vertical()
                                .max_height(400.0)
                                .show(ui, |ui| {
                                    JsonTree::new("json-body-tree", &json_value)
                                        .default_expand(default_expand)
                                        .show(ui);
                                });
                        });
                    return;
                }
                Err(_) => {
                    // If JSON parsing fails, fall through to plain text rendering
                }
            }
        }

        // Render as plain text
        egui::Frame::new()
            .fill(egui::Color32::from_gray(30))
            .inner_margin(10.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        let mut body_text = body;
                        ui.add(
                            egui::TextEdit::multiline(&mut body_text)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });
            });
    } else {
        ui.label(egui::RichText::new("(empty)").italics().weak());
    }
}

fn get_method_color(method: &str) -> egui::Color32 {
    match method {
        "GET" => egui::Color32::GREEN,
        "POST" => egui::Color32::BLUE,
        "PUT" => egui::Color32::YELLOW,
        "DELETE" => egui::Color32::RED,
        _ => egui::Color32::WHITE,
    }
}

fn generate_filename(timestamp: &str, path: &str, headers: &[(String, String)]) -> String {
    // Extract content-type from headers
    let content_type = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.as_str())
        .unwrap_or("text/plain");

    // Determine file extension based on content-type
    let extension = match content_type.split(';').next().unwrap_or("").trim() {
        "application/json" => "json",
        "application/xml" | "text/xml" => "xml",
        "text/html" => "html",
        "text/css" => "css",
        "text/javascript" | "application/javascript" => "js",
        "application/x-www-form-urlencoded" => "txt",
        "multipart/form-data" => "txt",
        "text/csv" => "csv",
        "application/pdf" => "pdf",
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        "application/zip" => "zip",
        "application/octet-stream" => "bin",
        _ => "txt",
    };

    // Clean up path to use as part of filename
    let path_clean = path
        .trim_start_matches('/')
        .replace(['/', '?', '&', '='], "_")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();

    // Truncate path if too long
    let path_part = if path_clean.len() > 50 {
        &path_clean[..50]
    } else {
        &path_clean
    };

    // Format timestamp for filename (YYYYMMDD_HHMMSS)
    let datetime = timestamp
        .replace(' ', "_")
        .replace([':', '.'], "")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>();

    // Combine parts: datetime_path.extension
    if path_part.is_empty() {
        format!("request_{}.{}", datetime, extension)
    } else {
        format!("{}_{}.{}", datetime, path_part, extension)
    }
}
