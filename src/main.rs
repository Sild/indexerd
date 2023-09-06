#[macro_use]
extern crate indexerd_derive;
extern crate hwloc;

mod objects;
mod store;
mod config;
mod engine;
mod db;
use hwloc::Topology;


use mysql_cdc::errors::Error;
mod server;



use crate::objects::MysqlObject;

fn check_cpu() {
    let topo = Topology::new();

    // Check if Process Binding for CPUs is supported
    println!("CPU Binding (current process) supported: {}", topo.support().cpu().set_current_process());
    println!("CPU Binding (any process) supported: {}", topo.support().cpu().set_process());
    // Check if Thread Binding for CPUs is supported
    println!("CPU Binding (current thread) supported: {}", topo.support().cpu().set_current_thread());
    println!("CPU Binding (any thread) supported: {}", topo.support().cpu().set_thread());

    // Debug Print all the Support Flags
    println!("All Flags:\n{:?}", topo.support());
}

fn main() -> Result<(), Error> {
    check_cpu();

    let c1 = objects::Campaign::from_select();
    let p1 = objects::Package::from_select();
    let pad1 = objects::Pad::from_select();

    let mut data_manager = store::data_manager::DataManager::default();
    data_manager.insert(c1);
    data_manager.insert(p1);
    data_manager.insert(pad1);

    let mut server = server::Server::default();
    server.run_admin_service(8089)?;
    server.init_engine();
    server.run_user_service(8088)?;
    println!("user service started");

    db::run_slave()
}