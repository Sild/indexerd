use crate::config;
use crate::data::objects::Storable;
use crate::data::objects::{Campaign, MysqlObject, Package, Pad};
use crate::data::updater;
use crate::data::updater::UpdaterPtr;
use logging_timer::stime;
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

#[stime("info")]
pub fn init(updater: &UpdaterPtr, db_conf: &config::DB) -> Result<(), Box<dyn Error>> {
    let mut conn = get_connection(db_conf)?;

    init_objects::<Campaign>(updater, &mut conn)?;
    init_objects::<Package>(updater, &mut conn)?;
    init_objects::<Pad>(updater, &mut conn)?;
    Ok(())
}

// select all objects from db and store type table id
fn init_objects<T>(updater: &UpdaterPtr, conn: &mut PooledConn) -> Result<(), mysql::Error>
where
    T: MysqlObject + FromRow + Storable + Default,
{
    let query = format!("SELECT * FROM {}", T::table());

    conn.query_map(query, |row| {
        let object = T::from_row(row);
        updater::apply_to_store(updater, object, None, updater::EventType::INSERT);
    })?;

    Ok(())
}

pub fn get_columns(db_conf: &config::DB, table: &str) -> Result<Vec<String>> {
    let mut conn = get_connection(db_conf)?;
    let columns = conn.query_map(
        format!("SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_NAME = '{}' ORDER BY ORDINAL_POSITION", table),
        |(name,)| name, // closure maps row to name
    )?;
    Ok(columns)
}
