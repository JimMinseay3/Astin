// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::{Command, Stdio};
use std::io::Write;
use std::path::PathBuf;

#[tauri::command]
async fn fetch_price(query: String, token: String) -> Result<serde_json::Value, String> {
    println!("Fetching security pool for: {} with token", query);
    
    let script_path = PathBuf::from("..").join("python_scripts").join("price_inquiry.py");
    
    let output = Command::new("..\\.venv\\Scripts\\python.exe")
        .arg(script_path)
        .arg("search")
        .arg(&query)
        .arg(&token)
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
async fn create_inquiry(body: String, token: String) -> Result<serde_json::Value, String> {
    println!("Creating new inquiry with token");
    
    let script_path = PathBuf::from("..").join("python_scripts").join("price_inquiry.py");
    
    let mut child = Command::new("..\\.venv\\Scripts\\python.exe")
        .arg(script_path)
        .arg("create")
        .arg(&token)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn python: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(body.as_bytes()).map_err(|e| format!("Failed to write to stdin: {}", e))?;
        // 显式关闭 stdin，通知 Python 数据发送完毕
        drop(stdin);
    }

    let output = child.wait_with_output().map_err(|e| format!("Failed to wait for python: {}", e))?;

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
async fn fetch_quote_list(page: u32, page_size: u32, security_code: String, token: String) -> Result<serde_json::Value, String> {
    println!("Fetching quote list: page={}, size={}, code={} with token", page, page_size, security_code);
    
    let script_path = PathBuf::from("..").join("python_scripts").join("price_inquiry.py");
    
    let output = Command::new("..\\.venv\\Scripts\\python.exe")
        .arg(script_path)
        .arg("list")
        .arg(page.to_string())
        .arg(page_size.to_string())
        .arg(&security_code)
        .arg(&token)
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
