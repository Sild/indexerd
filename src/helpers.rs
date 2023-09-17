use hwloc2::{CpuBindFlags, CpuSet, ObjectType, Topology};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub mod time {
    pub fn cur_ts() -> u64 {
        std::time::UNIX_EPOCH.elapsed().unwrap().as_secs()
    }
}

pub fn bind_thread(core_num: usize) -> Result<(), Box<dyn Error>> {
    let t_id = unsafe { libc::pthread_self() };
    let th = thread::current();
    let t_name = th.name().unwrap_or("empty");
    let mut topo = match Topology::new() {
        Some(t) => t,
        None => {
            return Err("fail to get topology".to_string().into());
        }
    };
    let before = topo.get_cpubind_for_thread(t_id, CpuBindFlags::CPUBIND_THREAD);

    let cpuset_to_bind = match cpuset_for_core(&topo, core_num) {
        Ok(c) => c,
        Err(e) => {
            return Err(e);
        }
    };

    topo.set_cpubind_for_thread(t_id, cpuset_to_bind, CpuBindFlags::CPUBIND_THREAD)
        .unwrap();
    let after = topo.get_cpubind_for_thread(t_id, CpuBindFlags::CPUBIND_THREAD);

    log::info!(
        "Bindind for thread_id={} (name='{}'), core={}: Before={:?}, After={:?}",
        t_id,
        t_name,
        core_num,
        before,
        after
    );
    Ok(())
}

fn cpuset_for_core(topology: &Topology, idx: usize) -> Result<CpuSet, Box<dyn Error>> {
    let cores = (*topology).objects_with_type(&ObjectType::PU).unwrap();
    match cores.get(idx) {
        Some(val) => Ok(val.cpuset().unwrap()),
        None => Err(format!("No Core found with id {}", idx).into()),
    }
}

pub struct ShutdownChecker {
    shutdown: Arc<AtomicBool>,
    last_check_ts: u64,
}

impl ShutdownChecker {
    pub fn new(shutdown: Arc<AtomicBool>) -> Self {
        ShutdownChecker {
            shutdown,
            last_check_ts: 0,
        }
    }

    pub fn check(&mut self) -> bool {
        self.check_impl(false)
    }

    pub fn check_force(&mut self) -> bool {
        self.check_impl(true)
    }

    fn check_impl(&mut self, force: bool) -> bool {
        let cur_ts = time::cur_ts();
        if cur_ts <= self.last_check_ts || force {
            return self.shutdown.load(Ordering::Relaxed);
        }
        self.last_check_ts = cur_ts;
        false
    }
}
