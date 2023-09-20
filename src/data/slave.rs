use crate::data::updater::Updater;
use crate::helpers::StopChecker;
use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_events::BinlogEvents;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use std::sync::{Arc, RwLock};

pub fn slave_loop(updater: Arc<RwLock<Updater>>) {
    let _options: BinlogOptions = BinlogOptions::from_start();
    let options: BinlogOptions = BinlogOptions::from_end();

    let (db_conf, shutdown) = match updater.read().expect("fail to get updater lock") {
        lock => (lock.conf.db.clone(), lock.stop_flag.clone()),
    };

    let options = ReplicaOptions {
        hostname: db_conf.host.clone(),
        port: db_conf.port,
        username: db_conf.username.clone(),
        password: db_conf.password.clone(),
        database: Some(db_conf.db_name.clone()),
        blocking: false,
        ssl_mode: SslMode::Disabled,
        binlog: options,
        ..Default::default()
    };

    let mut stop_checker = StopChecker::new(shutdown);
    let mut client = BinlogClient::new(options);
    while !stop_checker.is_time() {
        match client.replicate() {
            Ok(events) => {
                process_event(events, &mut stop_checker);
            }
            Err(e) => {
                log::warn!("Got error from slave stream: {:?}", e)
            }
        }
    }
    log::info!("slave thread finished");
}

fn process_event(events: BinlogEvents, sd_checker: &mut StopChecker) {
    for event in events {
        if sd_checker.is_time() {
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
