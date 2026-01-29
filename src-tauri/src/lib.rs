// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::Command;
use std::path::PathBuf;

#[tauri::command]
async fn fetch_price(query: String) -> Result<serde_json::Value, String> {
    println!("Fetching security pool for: {}", query);
    
    let script_path = PathBuf::from("..").join("python_scripts").join("price_inquiry.py");
    
    let output = Command::new("..\\.venv\\Scripts\\python.exe")
        .arg(script_path)
        .arg("search")
        .arg(&query)
        .output()
        .map_err(|e| format!("Failed to execute python: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse python output: {}", e))?;

    Ok(result)
}

#[tauri::command]
async fn create_inquiry(body: String) -> Result<serde_json::Value, String> {
    println!("Creating new inquiry");
    
    let script_path = PathBuf::from("..").join("python_scripts").join("price_inquiry.py");
    
    let output = Command::new("..\\.venv\\Scripts\\python.exe")
        .arg(script_path)
        .arg("create")
        .arg(&body)
        .output()
        .map_err(|e| format!("Failed to execute python: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse python output: {}", e))?;

    Ok(result)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PriceResult {
    site: String,
    price: String,
    stock: String,
    update_time: String,
}

#[tauri::command]
async fn fetch_quote_list(page: u32, page_size: u32, security_code: String) -> Result<serde_json::Value, String> {
    println!("Fetching quote list: page={}, size={}, code={}", page, page_size, security_code);
    
    let script_path = PathBuf::from("..").join("python_scripts").join("price_inquiry.py");
    
    let output = Command::new("..\\.venv\\Scripts\\python.exe")
        .arg(script_path)
        .arg("list")
        .arg(page.to_string())
        .arg(page_size.to_string())
        .arg(&security_code)
        .output()
        .map_err(|e| format!("Failed to execute python: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse python output: {}", e))?;

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![fetch_price, create_inquiry, fetch_quote_list])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
