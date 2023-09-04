#[macro_use]
extern crate indexerd_derive;
mod objects;
mod config;

use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::errors::Error;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;
use crate::config::DBConfig;

use crate::objects::MysqlObject;

fn run_slave(conf: DBConfig) -> Result<(), Error> {
    let _options: BinlogOptions = BinlogOptions::from_start();
    let options: BinlogOptions = BinlogOptions::from_end();

    let options = ReplicaOptions {
        port: conf.port,
        username: conf.username,
        password: conf.password,
        database: Some(conf.db_name),
        blocking: true,
        ssl_mode: SslMode::Disabled,
        binlog: options,
        ..Default::default()
    };

    let mut client = BinlogClient::new(options);

    for result in client.replicate()? {
        let (_header, event) = result?;
        // println!("{:#?}", header);
        // println!("{:#?}", event);
        match event {
            BinlogEvent::WriteRowsEvent(_) => {
                println!("write event")
            }
            _ => {println!("ignore event")}
        }

        // You process an event here

        // After you processed the event, you need to update replication position

    }
    Ok(())
}

fn main() -> Result<(), Error> {

    objects::campaign::Campaign::from_select();
    objects::campaign::Campaign::from_slave();
    println!("{}", objects::campaign::Campaign::table());

    let db_conf = match config::DBConfig::from_file("configs/dev.json") {
        Ok(conf) => conf,
        Err(e) => panic!("Fail to read config: {}", e)
    };
    println!("{:?}", db_conf);
    run_slave(db_conf)

}