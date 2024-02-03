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

pub fn bind_thread(core_num: usize) {
    // if cfg!(unix) {
    //     return;
    // }
    let t_id = unsafe { libc::pthread_self() };
    let th = thread::current();
    let t_name = th.name().unwrap_or("empty");
    let mut topo = match Topology::new() {
        Some(t) => t,
        None => {
            log::error!("fail to get topology");
            return;
        }
    };
    let before = topo.get_cpubind_for_thread(t_id, CpuBindFlags::CPUBIND_THREAD);

    let cpuset_to_bind = match cpuset_for_core(&topo, core_num) {
        Ok(c) => c,
        Err(e) => {
            log::error!("fail to get cpuset_to_bind, err={}", e);
            return;
        }
    };

    if let Err(e) = topo.set_cpubind_for_thread(t_id, cpuset_to_bind, CpuBindFlags::CPUBIND_THREAD)
    {
        log::error!(
            "fail to bind thread={} to core={}, err={:?}",
            t_name,
            core_num,
            e
        );
        return;
    }

    let after = topo.get_cpubind_for_thread(t_id, CpuBindFlags::CPUBIND_THREAD);

    log::info!(
        "Bindind for thread_name='{}', core={}: Before={:?}, After={:?}",
        t_name,
        core_num,
        before,
        after
    );
}

fn cpuset_for_core(topology: &Topology, idx: usize) -> Result<CpuSet, Box<dyn Error>> {
    let cores = (*topology).objects_with_type(&ObjectType::PU).unwrap();
    match cores.get(idx) {
        Some(val) => Ok(val.cpuset().unwrap()),
        None => Err(format!("No Core found with id {}", idx).into()),
    }
}

pub struct StopChecker {
    stop_flag: Arc<AtomicBool>,
    last_check_ts: u64,
}

impl StopChecker {
    pub fn new(shutdown: Arc<AtomicBool>) -> Self {
        StopChecker {
            stop_flag: shutdown,
            last_check_ts: 0,
        }
    }

    pub fn is_time(&mut self) -> bool {
        self.is_time_impl(false)
    }

    pub fn is_time_force(&mut self) -> bool {
        self.is_time_impl(true)
    }

    fn is_time_impl(&mut self, force: bool) -> bool {
        let cur_ts = time::cur_ts();
        if cur_ts > self.last_check_ts || force {
            return self.stop_flag.load(Ordering::Relaxed);
        }
        self.last_check_ts = cur_ts;
        false
    }
}
