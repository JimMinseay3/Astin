use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

#[tauri::command]
async fn fetch_price(app: tauri::AppHandle, query: String, token: String) -> Result<serde_json::Value, String> {
    let sidecar = app.shell().sidecar("price_inquiry")
        .map_err(|e| format!("无法创建 sidecar: {}", e))?;
    
    let output = sidecar
        .arg("search")
        .arg(&query)
        .arg(&token)
        .output()
        .await
        .map_err(|e| format!("执行失败: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python 脚本错误: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed_stdout = stdout.trim();
    
    if trimmed_stdout.is_empty() {
        return Err("Python 脚本没有产生任何输出".to_string());
    }

    let result: serde_json::Value = serde_json::from_str(trimmed_stdout)
        .map_err(|e| format!("解析输出失败: {}. 原始输出: [{}]", e, trimmed_stdout))?;

    Ok(result)
}

#[tauri::command]
async fn create_inquiry(app: tauri::AppHandle, body: String, token: String) -> Result<serde_json::Value, String> {
    let sidecar = app.shell().sidecar("price_inquiry")
        .map_err(|e| format!("无法创建 sidecar: {}", e))?;

    let (mut rx, mut child) = sidecar
        .arg("create")
        .arg(&token)
        .spawn()
        .map_err(|e| format!("启动失败: {}", e))?;

    let mut body_with_newline = body;
    body_with_newline.push('\n');
    
    child.write(body_with_newline.as_bytes()).map_err(|e| format!("写入 stdin 失败: {}", e))?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line) => stdout.extend(line),
            CommandEvent::Stderr(line) => stderr.extend(line),
            CommandEvent::Terminated(payload) => {
                if payload.code != Some(0) {
                    return Err(format!("Python 脚本错误 (退出码 {:?}): {}", payload.code, String::from_utf8_lossy(&stderr)));
                }
                break;
            }
            _ => {}
        }
    }

    let stdout_str = String::from_utf8_lossy(&stdout);
    let trimmed_stdout = stdout_str.trim();
    
    if trimmed_stdout.is_empty() {
        return Err("Python 脚本没有产生任何输出".to_string());
    }

    let result: serde_json::Value = serde_json::from_str(trimmed_stdout)
        .map_err(|e| format!("解析输出失败: {}. 原始输出: [{}]", e, trimmed_stdout))?;

    Ok(result)
}

#[tauri::command]
async fn fetch_quote_list(app: tauri::AppHandle, page: u32, page_size: u32, security_code: String, token: String) -> Result<serde_json::Value, String> {
    let sidecar = app.shell().sidecar("price_inquiry")
        .map_err(|e| format!("无法创建 sidecar: {}", e))?;

    let output = sidecar
        .arg("list")
        .arg(page.to_string())
        .arg(page_size.to_string())
        .arg(&security_code)
        .arg(&token)
        .output()
        .await
        .map_err(|e| format!("执行失败: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python 脚本错误: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed_stdout = stdout.trim();
    
    if trimmed_stdout.is_empty() {
        return Err("Python 脚本没有产生任何输出".to_string());
    }

    let result: serde_json::Value = serde_json::from_str(trimmed_stdout)
        .map_err(|e| format!("解析输出失败: {}. 原始输出: [{}]", e, trimmed_stdout))?;

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![fetch_price, create_inquiry, fetch_quote_list])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
