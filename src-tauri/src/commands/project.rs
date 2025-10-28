// Project command stubs - implement these according to tasks.md

#[tauri::command]
pub async fn create_new_project(name: String) -> Result<String, String> {
    // TODO: T104 - Implement project creation
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn save_project(path: String) -> Result<String, String> {
    // TODO: T102 - Implement project save
    Err("Not implemented yet".to_string())
}

#[tauri::command]
pub async fn load_project(path: String) -> Result<String, String> {
    // TODO: T103 - Implement project load
    Err("Not implemented yet".to_string())
}

