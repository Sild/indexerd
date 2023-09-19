use crate::config::Updater;
use crate::helpers::ShutdownChecker;
use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_events::BinlogEvents;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub fn run_slave(conf: &Updater, shutdown: Arc<AtomicBool>) {
    let _options: BinlogOptions = BinlogOptions::from_start();
    let options: BinlogOptions = BinlogOptions::from_end();

    let options = ReplicaOptions {
        hostname: conf.host.clone(),
        port: conf.port,
        username: conf.username.clone(),
        password: conf.password.clone(),
        database: Some(conf.db_name.clone()),
        blocking: false,
        ssl_mode: SslMode::Disabled,
        binlog: options,
        ..Default::default()
    };

    let mut sd_checker = ShutdownChecker::new(shutdown);
    let mut client = BinlogClient::new(options);
    loop {
        match client.replicate() {
            Ok(events) => {
                process_event(events, &mut sd_checker);
            }
            Err(e) => {
                log::warn!("Got error from slave stream: {:?}", e)
            }
        }
        if sd_checker.check() {
            break;
        }
    }
}

fn process_event(events: BinlogEvents, sd_checker: &mut ShutdownChecker) {
    for event in events {
        if sd_checker.check() {
            break;
        }
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
