use crate::config;
use crate::data::objects_traits::MysqlObject;
use crate::data::updater::{EventType, Updater, UpdaterPtr};
use crate::data::{objects, select, updater};
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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

#[derive(Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum SupportedTypes {
    Campaign,
    Package,
    Pad,
    PadRelation,
    TargetingPad,
    Unknown,
}

pub type FieldMapping = HashMap<String, usize>;

struct Context {
    updater: UpdaterPtr,
    table_id_map: HashMap<u64, SupportedTypes>,
    fields_map: HashMap<String, FieldMapping>,
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

    fill_fields_map::<objects::Campaign>(&db_conf, &mut ctx.fields_map, false);
    fill_fields_map::<objects::Package>(&db_conf, &mut ctx.fields_map, false);
    fill_fields_map::<objects::Pad>(&db_conf, &mut ctx.fields_map, false);

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
        log::trace!("got new slave event: '{:?}'", event);
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
            _ => log::trace!("ignore slave event with type={:?}", ev_type),
        }
        ctx.slave_cli.commit(&header, &ev_type);
    }
}

fn process_write(ctx: &mut Context, events: &WriteRowsEvent) {
    for ev in events.rows.iter() {
        if let Some(obj_type) = ctx.table_id_map.get(&events.table_id) {
            match obj_type {
                SupportedTypes::Campaign => {
                    updater::apply_to_store(
                        &ctx.updater,
                        objects::Campaign::from_slave(ev, &ctx.fields_map),
                        None,
                        EventType::INSERT,
                    );
                }
                SupportedTypes::Package => {
                    updater::apply_to_store(
                        &ctx.updater,
                        objects::Package::from_slave(ev, &ctx.fields_map),
                        None,
                        EventType::INSERT,
                    );
                }
                SupportedTypes::Pad => {
                    updater::apply_to_store(
                        &ctx.updater,
                        objects::Pad::from_slave(ev, &ctx.fields_map),
                        None,
                        EventType::INSERT,
                    );
                }
                SupportedTypes::PadRelation => {
                    updater::apply_to_store(
                        &ctx.updater,
                        objects::PadRelation::from_slave(ev, &ctx.fields_map),
                        None,
                        EventType::INSERT,
                    );
                }
                SupportedTypes::TargetingPad => {
                    updater::apply_to_store(
                        &ctx.updater,
                        objects::TargetingPad::from_slave(ev, &ctx.fields_map),
                        None,
                        EventType::INSERT,
                    );
                }
                SupportedTypes::Unknown => {}
            };
        }
    }
}

fn process_update(_ctx: &mut Context, events: &UpdateRowsEvent) {
    for ev in events.rows.iter() {}
}

fn process_delete(_ctx: &mut Context, events: &DeleteRowsEvent) {
    for ev in events.rows.iter() {}
}

fn process_table_map(ctx: &mut Context, event: &TableMapEvent) {
    if ctx.table_id_map.contains_key(&event.table_id) {
        return;
    }
    let t = serde_json::from_str::<SupportedTypes>(
        format!("\"{}\"", event.table_name.as_str()).as_str(),
    )
    .unwrap_or(SupportedTypes::Unknown);

    ctx.table_id_map.insert(event.table_id, t);
}

fn fill_fields_map<T: MysqlObject>(
    db_conf: &config::DB,
    fields_map: &mut HashMap<String, FieldMapping>,
    _retry: bool,
) {
    // todo: add retry logic
    let fields = match select::get_columns(db_conf, T::table()) {
        Ok(cols) => cols,
        Err(e) => {
            log::error!("fail to get columns for {}: {:?}", T::table(), e);
            return;
        }
    };

    let type_fields = FieldMapping::from_iter(
        fields
            .iter()
            .enumerate()
            .map(|(pos, field)| (field.clone(), pos)),
    );
    log::trace!(
        "fill_fields_map: table='{}', fields='{:?}'",
        T::table(),
        type_fields
    );
    fields_map.insert(T::table().into(), type_fields);
}
