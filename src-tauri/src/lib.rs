use tauri::Manager;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

fn get_python_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    
    // 1. 尝试从资源目录找（打包后的路径）
    let python_exe = resource_dir.join(".venv").join("Scripts").join("python.exe");
    if python_exe.exists() {
        return Ok(python_exe);
    }
    
    // 2. 尝试从当前工作目录找
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let dev_python = current_dir.join(".venv").join("Scripts").join("python.exe");
    if dev_python.exists() {
        return Ok(dev_python);
    }

    // 3. 尝试从上级目录找（Tauri 开发环境下 current_dir 可能是 src-tauri）
    if let Some(parent) = current_dir.parent() {
        let parent_python = parent.join(".venv").join("Scripts").join("python.exe");
        if parent_python.exists() {
            return Ok(parent_python);
        }
    }
    
    Err(format!("找不到 Python 解释器。已尝试路径:\n1. {:?}\n2. {:?}\n3. {:?}", 
        python_exe, 
        dev_python, 
        current_dir.parent().map(|p| p.join(".venv").join("Scripts").join("python.exe"))))
}

fn get_script_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    
    // 1. 尝试从资源目录找
    let script = resource_dir.join("python_scripts").join("price_inquiry.py");
    if script.exists() {
        return Ok(script);
    }
    
    // 2. 尝试从当前工作目录找
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    let dev_script = current_dir.join("python_scripts").join("price_inquiry.py");
    if dev_script.exists() {
        return Ok(dev_script);
    }

    // 3. 尝试从上级目录找
    if let Some(parent) = current_dir.parent() {
        let parent_script = parent.join("python_scripts").join("price_inquiry.py");
        if parent_script.exists() {
            return Ok(parent_script);
        }
    }

    Err(format!("找不到 Python 脚本。已尝试路径:\n1. {:?}\n2. {:?}\n3. {:?}", 
        script, 
        dev_script, 
        current_dir.parent().map(|p| p.join("python_scripts").join("price_inquiry.py"))))
}

#[tauri::command]
async fn fetch_price(app: tauri::AppHandle, query: String, token: String) -> Result<serde_json::Value, String> {
    let python_path = get_python_path(&app)?;
    let script_path = get_script_path(&app)?;

    let output = Command::new(python_path)
        .arg(script_path)
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
    
    // 如果输出为空，返回一个友好的错误
    if trimmed_stdout.is_empty() {
        return Err("Python 脚本没有产生任何输出".to_string());
    }

    let result: serde_json::Value = serde_json::from_str(trimmed_stdout)
        .map_err(|e| format!("解析输出失败: {}. 原始输出: [{}]", e, trimmed_stdout))?;

    Ok(result)
}

#[tauri::command]
async fn create_inquiry(app: tauri::AppHandle, body: String, token: String) -> Result<serde_json::Value, String> {
    let python_path = get_python_path(&app)?;
    let script_path = get_script_path(&app)?;

    let mut child = Command::new(python_path)
        .arg(script_path)
        .arg("create")
        .arg(&token)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动失败: {}", e))?;

    let mut stdin = child.stdin.take().ok_or("无法打开 stdin")?;
    
    let mut body_with_newline = body;
    body_with_newline.push('\n');
    
    stdin.write_all(body_with_newline.as_bytes()).await.map_err(|e| format!("写入 stdin 失败: {}", e))?;
    drop(stdin); // 显式关闭 stdin 以发送 EOF

    let output = child.wait_with_output().await.map_err(|e| format!("等待输出失败: {}", e))?;

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
async fn fetch_quote_list(app: tauri::AppHandle, page: u32, page_size: u32, security_code: String, token: String) -> Result<serde_json::Value, String> {
    let python_path = get_python_path(&app)?;
    let script_path = get_script_path(&app)?;

    let output = Command::new(python_path)
        .arg(script_path)
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
