use crate::config;
use crate::data::updater::Updater;
use crate::helpers::StopChecker;
use mysql::prelude::*;
use mysql::*;
use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_events::BinlogEvents;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub fn get_master_gtid(db_conf: &config::DB) -> Result<Option<GtidSet>, Box<dyn Error>> {
    let url = format!(
        "mysql://{}:{}@{}:{}/",
        db_conf.username, db_conf.password, db_conf.host, db_conf.port
    );
    let pool = Pool::new(url.as_str())?;
    let mut conn = pool.get_conn()?;

    let gtid: Option<String> = conn.query_first("SELECT @@global.gtid_executed")?;
    if let Some(gtid) = gtid {
        return match GtidSet::parse(gtid.as_str()) {
            Ok(gtid_parsed) => Ok(Some(gtid_parsed)),
            Err(e) => {
                log::error!("fail to parse gtid: {:?}", e);
                Ok(None)
            }
        };
    }
    Ok(None)
}

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
            Ok(e) => e,
            Err(e) => {
                log::warn!("fail extract event, err={:?}", e);
                continue;
            }
        };
        match ev_type {
            BinlogEvent::WriteRowsEvent(_) => {
                log::debug!("write event")
            }
            _ => {
                log::trace!("ignore event: {:?}", ev_type)
            }
        }
        client.commit(&header, &ev_type);
    }
}
