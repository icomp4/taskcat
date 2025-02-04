use sysinfo::{System, Pid, Process};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct ProcessItem {
    id: u32,
    name: String,
    cpu_usage: f32,
    memory_usage: f64
}

#[derive(Serialize)]
struct ProcessGroup {
    id: u32,
    name: String,
    cpu_usage: f32,
    memory_usage: f64,
    children: Vec<ProcessItem>,
    total_cpu: f32,
    total_memory: f64
}

#[tauri::command]
fn get_processes() -> Vec<ProcessGroup> {
    let mut s = System::new_all();
    s.refresh_all();

    let mut groups: HashMap<Option<Pid>, Vec<(Pid, &Process)>> = HashMap::new();
    for (pid, process) in s.processes() {
        groups.entry(process.parent())
            .or_default()
            .push((*pid, process));
    }

    groups.into_iter()
        .filter_map(|(parent_pid, processes)| {
            if let Some(parent) = parent_pid.and_then(|ppid| s.process(ppid)) {
                let children: Vec<ProcessItem> = processes.iter()
                    .map(|(pid, proc)| ProcessItem {
                        id: pid.as_u32(),
                        name: proc.name().to_string_lossy().into_owned(),
                        cpu_usage: proc.cpu_usage(),
                        memory_usage: (proc.memory() as f64 / 1000000.0).round()
                    })
                    .collect();

                Some(ProcessGroup {
                    id: parent_pid?.as_u32(),
                    name: parent.name().to_string_lossy().into_owned(),
                    cpu_usage: parent.cpu_usage(),
                    memory_usage: (parent.memory() as f64 / 1000000.0).round(),
                    total_cpu: children.iter().map(|p| p.cpu_usage).sum(),
                    total_memory: children.iter().map(|p| p.memory_usage).sum(),
                    children
                })
            } else {
                None
            }
        })
        .collect()
}

#[tauri::command]
async fn kill_process(pid: u32) -> Result<(), String> {
    let mut s = System::new_all();
    s.refresh_all();

    let process = s.process(Pid::from(pid as usize))
        .ok_or_else(|| format!("Process {} not found", pid))?;

    if !process.kill() {
        return Err(format!("Failed to kill process {}", pid));
    }
    Ok(())
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
        get_processes,
        kill_process
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
