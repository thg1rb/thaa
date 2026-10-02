use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoDto {
    product_name: &'static str,
    version: &'static str,
}

#[tauri::command]
pub fn get_app_info() -> AppInfoDto {
    AppInfoDto {
        product_name: "Thaa — Local Runtime Inspector",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg(test)]
mod tests {
    use super::get_app_info;
    use serde_json::json;

    #[test]
    fn app_info_uses_the_frontend_transport_shape() {
        let response = serde_json::to_value(get_app_info()).expect("DTO should serialize");

        assert_eq!(
            response,
            json!({
                "productName": "Thaa — Local Runtime Inspector",
                "version": env!("CARGO_PKG_VERSION")
            })
        );
    }
}
