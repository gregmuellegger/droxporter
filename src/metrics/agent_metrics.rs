use std::sync::Arc;
use parking_lot::Mutex;
use prometheus::{Gauge, Registry};
use sysinfo::{ProcessExt, System, SystemExt};

pub trait AgentMetricsService: Send + Sync {
    fn load_agent_metrics(&self) -> anyhow::Result<()>;
}


#[derive(Clone)]
pub struct AgentMetricsImpl {
    system: Arc<Mutex<sysinfo::System>>,
    cpu: Gauge,
    memory: Gauge,
}

impl AgentMetricsImpl {
    pub fn new(registry: Registry) -> Self {
        let system = System::new();
        let cpu = Gauge::new("droxporter_self_cpu_usage", "CPU usage of DO Loading agent").unwrap();
        let memory = Gauge::new("droxporter_self_memory_usage", "CPU usage of DO Loading agent").unwrap();
        registry.register(Box::new(cpu.clone())).unwrap();
        registry.register(Box::new(memory.clone())).unwrap();
        Self {
            system: Arc::new(Mutex::new(system)),
            cpu,
            memory,
        }
    }
}


impl AgentMetricsService for AgentMetricsImpl {
    fn load_agent_metrics(&self) -> anyhow::Result<()> {
        let mut system = self.system.lock();
        system.refresh_all();

        let pid = sysinfo::get_current_pid()
            .map_err(|s| anyhow::Error::msg(s))?;
        let process = system.process(pid)
            .ok_or(anyhow::Error::msg("Process not found"))?;

        self.cpu.set(process.cpu_usage() as f64);
        self.memory.set((process.memory()) as f64 * 1024.0);
        Ok(())
    }
}