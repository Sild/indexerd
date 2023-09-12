use crate::config::DBConfig;
use crate::helpers;
use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_events::BinlogEvents;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::errors::Error;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub fn run_slave(shutdown: Arc<AtomicBool>) -> Result<(), Error> {
    let conf = match DBConfig::from_file("configs/dev.json") {
        Ok(conf) => conf,
        Err(e) => panic!("Fail to read config: {}", e),
    };
    log::info!("{:?}", conf);
    let _options: BinlogOptions = BinlogOptions::from_start();
    let options: BinlogOptions = BinlogOptions::from_end();

    let options = ReplicaOptions {
        port: conf.port,
        username: conf.username,
        password: conf.password,
        database: Some(conf.db_name),
        blocking: false,
        ssl_mode: SslMode::Disabled,
        binlog: options,
        ..Default::default()
    };

    let mut _sd_checker = helpers::ShutdownChecker::new(shutdown);
    let mut client = BinlogClient::new(options);
    loop {
        match client.replicate() {
            Ok(events) => {
                process_event(events);
            }
            Err(_) => {
                log::warn!("Got error from slabe stream")
            }
        }
        // if sd_checker.check() {
        //     break;
        // }
    }
    Ok(())
}

fn process_event(events: BinlogEvents) {
    for event in events {
        let (_header, ev_type) = event.expect("got event with error");
        match ev_type {
            BinlogEvent::WriteRowsEvent(_) => {
                log::trace!("write event")
            }
            _ => {
                log::trace!("ignore event: {:?}", ev_type)
            }
        }
    }
}
