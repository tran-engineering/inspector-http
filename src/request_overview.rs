use crate::HttpRequest;
use eframe::egui;

pub fn render_request_overview(
    ui: &mut egui::Ui,
    requests: &[HttpRequest],
    selected_request: &mut Option<usize>,
    server_port: u16,
) -> bool {
    let mut clear_requests = false;

    ui.horizontal(|ui| {
        ui.heading("Requests");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if !requests.is_empty() {
                if ui.button("🗑 Clear All").clicked() {
                    clear_requests = true;
                }
            }
        });
    });
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Show helpful message when no requests yet
        if requests.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No requests yet")
                        .size(18.0)
                        .color(egui::Color32::GRAY),
                );
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Send a request to get started:")
                        .size(14.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(10.0);

                // Show curl command
                let curl_command = format!("curl http://localhost:{}", server_port);
                egui::Frame::new()
                    .fill(egui::Color32::from_gray(30))
                    .inner_margin(10.0)
                    .corner_radius(5.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&curl_command)
                                    .monospace()
                                    .color(egui::Color32::LIGHT_GREEN),
                            );
                            if ui
                                .button("📋")
                                .on_hover_text("Copy to clipboard")
                                .clicked()
                            {
                                ui.ctx().copy_text(curl_command.clone());
                            }
                        });
                    });

                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Or send a POST request:")
                        .size(12.0)
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.add_space(5.0);

                let post_command = format!(
                    "curl -X POST http://localhost:{} -d '{{\"key\":\"value\"}}'",
                    server_port
                );
                egui::Frame::new()
                    .fill(egui::Color32::from_gray(30))
                    .inner_margin(10.0)
                    .corner_radius(5.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&post_command)
                                    .monospace()
                                    .color(egui::Color32::LIGHT_GREEN),
                            );
                            if ui
                                .button("📋")
                                .on_hover_text("Copy to clipboard")
                                .clicked()
                            {
                                ui.ctx().copy_text(post_command.clone());
                            }
                        });
                    });
            });
        }

        for (idx, req) in requests.iter().enumerate().rev() {
            let actual_idx = requests.len() - 1 - (requests.len() - 1 - idx);
            let is_selected = *selected_request == Some(actual_idx);

            let response = ui.selectable_label(
                is_selected,
                egui::RichText::new(format!(
                    "{} {} {}",
                    req.timestamp.split_whitespace().nth(1).unwrap_or(""),
                    req.method,
                    req.path
                )),
            );

            if response.clicked() {
                *selected_request = Some(actual_idx);
            }

            // Show method color indicator and basic info
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(&req.method)
                        .small()
                        .color(get_method_color(&req.method)),
                );
                ui.label(egui::RichText::new(&req.remote_addr).small().weak());
                if !req.query_params.is_empty() {
                    ui.label(
                        egui::RichText::new(format!("{} params", req.query_params.len()))
                            .small()
                            .weak(),
                    );
                }
                if req.body_size > 0 {
                    ui.label(
                        egui::RichText::new(format!("{} bytes", req.body_size))
                            .small()
                            .weak(),
                    );
                }
            });

            ui.separator();
        }
    });

    clear_requests
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
