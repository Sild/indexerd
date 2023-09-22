use crate::config;
use crate::data::objects::MysqlObject;
use crate::data::slave::SupportedTypes::Campaign;
use crate::data::updater::{EventType, Updater, UpdaterPtr};
use crate::data::{objects, select};
use crate::helpers::StopChecker;
use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_events::BinlogEvents;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::events::row_events::delete_rows_event::DeleteRowsEvent;
use mysql_cdc::events::row_events::update_rows_event::UpdateRowsEvent;
use mysql_cdc::events::row_events::write_rows_event::WriteRowsEvent;
use mysql_cdc::events::table_map_event::TableMapEvent;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, Hash, Eq, PartialEq)]
enum SupportedTypes {
    Campaign,
    Package,
    Pad,
    Unknown,
    // PadRelation,
    // TargetingPad,
}

pub type FieldMapping = HashMap<String, u32>;

struct Context {
    updater: UpdaterPtr,
    table_id_map: HashMap<u64, SupportedTypes>,
    fields_map: HashMap<SupportedTypes, FieldMapping>,
    slave_cli: BinlogClient,
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

    let mut ctx = Context {
        updater,
        table_id_map: HashMap::new(),
        fields_map: HashMap::new(),
        slave_cli: BinlogClient::new(build_slave_cli_opts(&db_conf, start_gtid)),
    };

    let mut campaign_map = FieldMapping::new();
    let mut campaign_fields = match select::get_columns(&db_conf, objects::Campaign::table()) {
        Ok(f) => f,
        Err(e) => {
            log::error!("error getting campaign fields: {}", e);
            return;
        }
    };
    for (pos, name) in campaign_fields.iter().enumerate() {
        campaign_map.insert(name.clone(), pos as u32);
    }
    ctx.fields_map.insert(Campaign, FieldMapping::new());

    let mut stop_checker = StopChecker::new(stop_flag);

    while !stop_checker.is_time() {
        match ctx.slave_cli.replicate() {
            Ok(events) => {
                process_events(&mut ctx, events);
            }
            Err(e) => {
                log::warn!("Got error from slave stream: {:?}", e)
            }
        }
    }
    log::info!("slave thread finished");
}

fn process_events(ctx: &mut Context, events: BinlogEvents) {
    for event in events {
        let (header, ev_type) = match event {
            Ok(ev) => ev,
            Err(err) => {
                log::warn!("fail to extract event, err={:?}", err);
                continue;
            }
        };
        match ev_type {
            BinlogEvent::WriteRowsEvent(ref ev_body) => process_write(ctx, ev_body),
            BinlogEvent::UpdateRowsEvent(ref ev_body) => process_update(ctx, ev_body),
            BinlogEvent::DeleteRowsEvent(ref ev_body) => process_delete(ctx, ev_body),
            BinlogEvent::TableMapEvent(ref ev_body) => process_table_map(ctx, ev_body),
            _ => log::trace!("ignore event: {:?}", ev_type),
        }
        ctx.slave_cli.commit(&header, &ev_type);
    }
}

fn process_write(ctx: &mut Context, events: &WriteRowsEvent) {
    log::info!("table_id: {}, events: {:?}", events.table_id, events);

    for ev in events.rows.iter() {
        log::info!("table_id: {}, write_ev: {:?}", events.table_id, ev);
        // let obj = match &ctx.table_id_map[&events.table_id] {
        //     SupportedTypes::Campaign => Campaign::from_row(ev.into()),
        //     SupportedTypes::Package => Package::from_row(ev.into()),
        //     SupportedTypes::Pad => Pad::from_row(ev.into()),
        //     _ => Campaign::default(),
        // };
        // updater::apply_to_store(&ctx.updater, obj, None, EventType::INSERT)
    }
}

fn process_update(ctx: &mut Context, events: &UpdateRowsEvent) {
    for ev in events.rows.iter() {
        log::info!("table_id: {}, update_ev: {:?}", events.table_id, ev)
    }
}

fn process_delete(ctx: &mut Context, events: &DeleteRowsEvent) {
    for ev in events.rows.iter() {
        log::info!("table_id: {}, delete_ev: {:?}", events.table_id, ev)
    }
}

fn process_table_map(ctx: &mut Context, event: &TableMapEvent) {
    if ctx.table_id_map.contains_key(&event.table_id) {
        return;
    }
    ctx.table_id_map.insert(
        event.table_id,
        match event.table_name.as_str() {
            "campaign" => SupportedTypes::Campaign,
            "package" => SupportedTypes::Package,
            "pad" => SupportedTypes::Pad,
            _ => SupportedTypes::Unknown,
        },
    );
    log::info!("table_map_event: {:?}", event)
}
