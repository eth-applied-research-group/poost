//! Collects and caches host hardware / OS data for the "/info" route.

use axum::Json;
use serde::Serialize;
use sysinfo::System;
use tracing::instrument;
use wgpu::{Backends, Instance, InstanceDescriptor};

#[derive(Debug, Serialize)]
pub struct ServerInfoResponse {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub os: OsInfo,
    pub architecture: String,
    pub gpu: String,
}

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub frequency: u64,
    pub vendor: String,
}

#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    pub total: String,
    pub available: String,
    pub used: String,
}

#[derive(Debug, Serialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub kernel: String,
}

/// Get informatio about the CPU
/// Note: unfortunately this has been unreliable on ARM macs and AWS machines
fn get_cpu_info() -> CpuInfo {
    let mut sys = System::new_all();
    sys.refresh_cpu();
    let cpu = sys.global_cpu_info();

    CpuInfo {
        model: cpu.brand().to_string(),
        cores: sys.physical_core_count().unwrap_or(0),
        frequency: cpu.frequency(),
        vendor: cpu.vendor_id().to_string(),
    }
}

/// Get memory related information about the system
/// TODO:: I think just having total is likely fine
fn get_memory_info() -> MemoryInfo {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total = sys.total_memory();
    let available = sys.available_memory();
    let used = total - available;

    MemoryInfo {
        total: format!("{:.2} GB", total as f64 / 1024.0 / 1024.0 / 1024.0),
        available: format!("{:.2} GB", available as f64 / 1024.0 / 1024.0 / 1024.0),
        used: format!("{:.2} GB", used as f64 / 1024.0 / 1024.0 / 1024.0),
    }
}

/// Get OS specific information
fn get_os_info() -> OsInfo {
    OsInfo {
        name: System::name().unwrap_or_else(|| "Unknown".into()),
        version: System::os_version().unwrap_or_else(|| "Unknown".into()),
        kernel: System::kernel_version().unwrap_or_else(|| "Unknown".into()),
    }
}

/// Get GPU specific information
fn get_gpu_info() -> String {
    let instance_desc = InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    };

    let instance = Instance::new(&instance_desc);
    let names: Vec<String> = instance
        .enumerate_adapters(Backends::all())
        .into_iter()
        .map(|a| a.get_info().name)
        .collect();

    match names.as_slice() {
        [] => "No GPU detected".into(),
        [single] => single.clone(),
        many => many.join(" · "),
    }
}

#[instrument]
pub async fn get_server_info() -> Json<ServerInfoResponse> {
    Json(ServerInfoResponse {
        cpu: get_cpu_info(),
        memory: get_memory_info(),
        os: get_os_info(),
        architecture: std::env::consts::ARCH.into(),
        gpu: get_gpu_info(),
    })
}
#[cfg(test)]
mod tests {
    use crate::endpoints::get_server_info;

    #[tokio::test]
    async fn test_server_info() {
        let info = get_server_info().await;
        // TODO: check why cpu_model is empty
        // assert!(!info.0.cpu.model.is_empty());
        assert!(!info.0.memory.total.is_empty());
        assert!(!info.0.os.name.is_empty());
        assert!(!info.0.architecture.is_empty());
        assert!(!info.0.gpu.is_empty());
    }
}
