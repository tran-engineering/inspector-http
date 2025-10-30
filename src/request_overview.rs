use crate::HttpRequest;
use eframe::egui;

pub fn render_request_overview(
    ui: &mut egui::Ui,
    requests: &[HttpRequest],
    selected_request: &mut Option<usize>,
) {
    ui.heading("Requests");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
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
