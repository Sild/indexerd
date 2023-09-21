use crate::config;
use crate::data::objects::Storable;
use crate::data::objects::{Campaign, MysqlObject, Package, Pad};
use crate::data::updater;
use crate::data::updater::UpdaterPtr;
use mysql::prelude::*;
use mysql::*;
use mysql_cdc::providers::mysql::gtid::gtid_set::GtidSet;
use std::error::Error;

pub fn get_connection(db_conf: &config::DB) -> Result<PooledConn> {
    let url = format!(
        "mysql://{}:{}@{}:{}/{}",
        db_conf.username, db_conf.password, db_conf.host, db_conf.port, db_conf.db_name
    );
    let pool = Pool::new(url.as_str())?;
    pool.get_conn()
}

// used once for store initialization
pub fn get_master_gtid(db_conf: &config::DB) -> Result<Option<GtidSet>, Box<dyn Error>> {
    let mut conn = get_connection(db_conf)?;

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

pub fn init(updater: &UpdaterPtr) -> Result<(), Box<dyn Error>> {
    let db_conf = updater.read().unwrap().conf.db.clone();
    let mut conn = get_connection(&db_conf)?;
    select_objects::<Campaign>(&updater, &mut conn)?;
    select_objects::<Package>(&updater, &mut conn)?;
    select_objects::<Pad>(&updater, &mut conn)?;
    Ok(())
}

fn select_objects<T: MysqlObject + FromRow + Storable>(
    updater: &UpdaterPtr,
    conn: &mut PooledConn,
) -> Result<(), mysql::Error> {
    let query = format!("SELECT * FROM {}", T::table());

    conn.query_map(query, |row| {
        let object = T::from_row(row);
        updater::apply_to_store(&updater, object, None, updater::EventType::INSERT);
    })?;

    Ok(())
}
