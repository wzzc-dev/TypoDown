// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    use std::fs;

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(content)
}

#[tauri::command]
async fn write_file(path: String, content: String) -> Result<(), String> {
    use std::fs;

    fs::write(&path, &content)
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn open_file_dialog() -> Result<Option<String>, String> {
    use rfd::FileDialog;

    let file_path = FileDialog::new()
        .add_filter("Markdown", &["md", "markdown"])
        .pick_file()
        .map(|path| path.to_string_lossy().to_string());

    Ok(file_path)
}

#[tauri::command]
async fn save_file_dialog(default_name: String) -> Result<Option<String>, String> {
    use rfd::FileDialog;

    let file_path = FileDialog::new()
        .set_file_name(&default_name)
        .add_filter("Markdown", &["md", "markdown"])
        .save_file()
        .map(|path| path.to_string_lossy().to_string());

    Ok(file_path)
}

#[tauri::command]
async fn read_dir(path: String) -> Result<Vec<String>, String> {
    use std::fs;
    use std::path::Path;

    let entries = fs::read_dir(&path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut files: Vec<String> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path().to_string_lossy().to_string())
        .collect();

    files.sort_by(|a, b| {
        let path_a = Path::new(a);
        let path_b = Path::new(b);

        let is_dir_a = path_a.is_dir();
        let is_dir_b = path_b.is_dir();

        if is_dir_a && !is_dir_b {
            std::cmp::Ordering::Less
        } else if !is_dir_a && is_dir_b {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    Ok(files)
}

#[tauri::command]
fn home_dir() -> Result<String, String> {
    let home = dirs::home_dir()
        .ok_or_else(|| "Failed to get home directory".to_string())?;
    Ok(home.to_string_lossy().to_string())
}

#[tauri::command]
fn documents_dir() -> Result<Option<String>, String> {
    let docs = dirs::document_dir()
        .map(|path| path.to_string_lossy().to_string());
    Ok(docs)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            read_file,
            write_file,
            open_file_dialog,
            save_file_dialog,
            read_dir,
            home_dir,
            documents_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
