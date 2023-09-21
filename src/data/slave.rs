use crate::config;
use crate::data::updater::Updater;
use crate::helpers::StopChecker;
use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_events::BinlogEvents;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use std::sync::{Arc, RwLock};
use std::time::Duration;

fn build_slave_cli_opts(db_conf: &config::DB, gtid: Option<GtidSet>) -> ReplicaOptions {
    ReplicaOptions {
        hostname: db_conf.host.clone(),
        port: db_conf.port,
        username: db_conf.username.clone(),
        password: db_conf.password.clone(),
        database: Some(db_conf.db_name.clone()),
        server_id: 65535,
        blocking: false,
        ssl_mode: SslMode::Disabled,
        binlog: match gtid {
            Some(g) => BinlogOptions::from_mysql_gtid(g),
            None => BinlogOptions::from_start(),
        },
        heartbeat_interval: Duration::from_secs(5),
    }
}

pub fn slave_loop(updater: Arc<RwLock<Updater>>, start_gtid: Option<GtidSet>) {
    let (db_conf, stop_flag) = match updater.read().expect("fail to get updater lock") {
        lock => (lock.conf.db.clone(), lock.stop_flag.clone()),
    };

    let mut slave_cli = BinlogClient::new(build_slave_cli_opts(&db_conf, start_gtid));

    let mut stop_checker = StopChecker::new(stop_flag);

    while !stop_checker.is_time() {
        match slave_cli.replicate() {
            Ok(events) => {
                process_events(&mut slave_cli, events, &mut stop_checker);
            }
            Err(e) => {
                log::warn!("Got error from slave stream: {:?}", e)
            }
        }
    }
    // client.
    log::info!("slave thread finished");
}

fn process_events(client: &mut BinlogClient, events: BinlogEvents, sd_checker: &mut StopChecker) {
    for event in events {
        if sd_checker.is_time() {
            break;
        }
        let (header, ev_type) = match event {
            Ok(ev) => ev,
            Err(err) => {
                log::warn!("fail extract event, err={:?}", err);
                continue;
            }
        };
        match ev_type {
            BinlogEvent::WriteRowsEvent(ref ev_body) => {
                log::debug!("write event: header={:?}, ev_body={:?}", header, ev_body);
            }
            _ => {
                log::trace!("ignore event: {:?}", ev_type)
            }
        }
        client.commit(&header, &ev_type);
    }
}

#[allow(dead_code)]
fn process_event_row(
    _client: &mut BinlogClient,
    _event: BinlogEvents,
    _sd_checker: &mut StopChecker,
) {
}
