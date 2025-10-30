use eframe::egui;

#[derive(Clone, Debug)]
pub struct ResponseConfig {
    pub status_code: u16,
    pub status_code_input: String,
    pub response_body: String,
}

impl Default for ResponseConfig {
    fn default() -> Self {
        Self {
            status_code: 200,
            status_code_input: "200".to_string(),
            response_body: "OK\n".to_string(),
        }
    }
}

pub fn render_response_config(ui: &mut egui::Ui, config: &mut ResponseConfig) {
    ui.heading("Response Configuration");
    ui.separator();

    egui::ScrollArea::both().show(ui, |ui| {
        ui.add_space(10.0);

        // HTTP Status Code section
        ui.label(egui::RichText::new("HTTP Status Code").heading());
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Status Code:");
            let status_edit = egui::TextEdit::singleline(&mut config.status_code_input)
                .desired_width(80.0)
                .hint_text("200");

            if ui.add(status_edit).changed() {
                // Try to parse and validate the status code
                if let Ok(code) = config.status_code_input.parse::<u16>()
                    && (100..=599).contains(&code)
                {
                    config.status_code = code;
                }
            }

            // Show status code description
            ui.label(egui::RichText::new(get_status_description(config.status_code)).weak());
        });

        ui.add_space(5.0);

        // Quick selection buttons for common status codes
        ui.label(egui::RichText::new("Quick Select:").small());
        ui.horizontal_wrapped(|ui| {
            let common_codes = [
                (200, "200 OK"),
                (201, "201 Created"),
                (204, "204 No Content"),
                (400, "400 Bad Request"),
                (401, "401 Unauthorized"),
                (403, "403 Forbidden"),
                (404, "404 Not Found"),
                (500, "500 Internal Server Error"),
                (502, "502 Bad Gateway"),
                (503, "503 Service Unavailable"),
            ];

            for (code, label) in &common_codes {
                if ui.button(*label).clicked() {
                    config.status_code = *code;
                    config.status_code_input = code.to_string();
                }
            }
        });

        ui.add_space(20.0);

        // Response Body section
        ui.label(egui::RichText::new("Response Body").heading());
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label(format!("Body size: {} bytes", config.response_body.len()));

            if ui.button("Clear").clicked() {
                config.response_body.clear();
            }
        });

        ui.add_space(5.0);

        // Body text editor
        egui::Frame::new()
            .fill(egui::Color32::from_gray(30))
            .inner_margin(10.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut config.response_body)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(15)
                                .hint_text("Enter response body here..."),
                        );
                    });
            });

        ui.add_space(10.0);

        // Quick templates
        ui.label(egui::RichText::new("Quick Templates:").small());
        ui.horizontal_wrapped(|ui| {
            if ui.button("Empty").clicked() {
                config.response_body.clear();
            }
            if ui.button("OK").clicked() {
                config.response_body = "OK\n".to_string();
            }
            if ui.button("JSON Success").clicked() {
                config.response_body = r#"{
  "status": "success",
  "message": "Request processed successfully"
}
"#
                .to_string();
            }
            if ui.button("JSON Error").clicked() {
                config.response_body = r#"{
  "status": "error",
  "message": "An error occurred",
  "code": "ERROR_CODE"
}
"#
                .to_string();
            }
            if ui.button("HTML").clicked() {
                config.response_body = r#"<!DOCTYPE html>
<html>
<head>
    <title>Response</title>
</head>
<body>
    <h1>Hello from Inspector HTTP</h1>
    <p>This is a custom response.</p>
</body>
</html>
"#
                .to_string();
            }
        });

        ui.add_space(20.0);

        // Info box
        ui.separator();
        ui.add_space(5.0);
        ui.label(
            egui::RichText::new(
                "ℹ All incoming HTTP requests will receive this configured response.",
            )
            .small()
            .color(egui::Color32::LIGHT_BLUE),
        );
    });
}

fn get_status_description(code: u16) -> &'static str {
    match code {
        // 1xx Informational
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",

        // 2xx Success
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",

        // 3xx Redirection
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",

        // 4xx Client Errors
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        422 => "Unprocessable Entity",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",

        // 5xx Server Errors
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",

        _ => "Unknown Status Code",
    }
}
