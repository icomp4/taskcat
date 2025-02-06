use sysinfo::{System, Pid, Process, CpuRefreshKind, RefreshKind};
use serde::Serialize;
use std::collections::HashMap;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

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

#[derive(Serialize)]
struct TotalUsage {
    cpu: f32,
    memory: u64
}

#[tauri::command]
fn get_processes() -> Vec<ProcessGroup> {
   let mut s = System::new_with_specifics(
       RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
   );
   std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
   s.refresh_cpu_usage();
   s.refresh_all();

   let total_cpu_time = s.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>();

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
                       cpu_usage: if total_cpu_time > 0.0 {
                           Decimal::from_f32(proc.cpu_usage() / total_cpu_time * 100.0)
                               .unwrap_or_default()
                               .round_dp(2)
                               .to_f32()
                               .unwrap_or(0.0)
                       } else {
                           0.0
                       },
                       memory_usage: Decimal::from_f64(proc.memory() as f64 / 1000000.0)
                           .unwrap_or_default()
                           .round_dp(2)
                           .to_f64()
                           .unwrap_or(0.0)
                   })
                   .collect();

               let parent_cpu = if total_cpu_time > 0.0 {
                   Decimal::from_f32(parent.cpu_usage() / total_cpu_time * 100.0)
                       .unwrap_or_default()
                       .round_dp(2)
                       .to_f32()
                       .unwrap_or(0.0)
               } else {
                   0.0
               };

               Some(ProcessGroup {
                   id: parent_pid?.as_u32(),
                   name: parent.name().to_string_lossy().into_owned(),
                   cpu_usage: parent_cpu,
                   memory_usage: Decimal::from_f64(parent.memory() as f64 / 1000000.0)
                       .unwrap_or_default()
                       .round_dp(2)
                       .to_f64()
                       .unwrap_or(0.0),
                   total_cpu: Decimal::from_f32(children.iter().map(|p| p.cpu_usage).sum())
                       .unwrap_or_default()
                       .round_dp(2)
                       .to_f32()
                       .unwrap_or(0.0),
                   total_memory: Decimal::from_f64(children.iter().map(|p| p.memory_usage).sum())
                       .unwrap_or_default()
                       .round_dp(2)
                       .to_f64()
                       .unwrap_or(0.0),
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

#[tauri::command]
fn get_total_usage() -> TotalUsage {
    let mut s = System::new_with_specifics(
        RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
    );
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    s.refresh_cpu_usage();

    let cpu = s.global_cpu_usage();
    let mut sys = System::new_all();
    sys.refresh_all();

    let total_cpu = Decimal::from_f32(cpu)
        .unwrap_or_default()
        .round_dp(2)
        .to_f32()
        .unwrap_or(0.0);

    TotalUsage{
        cpu: total_cpu,
        memory: sys.used_memory() / 1000000
    }
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .invoke_handler(tauri::generate_handler![
        get_processes,
        kill_process,
        get_total_usage
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
