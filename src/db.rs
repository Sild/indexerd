use crate::config::DBConfig;
use log::log;
use mysql_cdc::binlog_client::BinlogClient;
use mysql_cdc::binlog_options::BinlogOptions;
use mysql_cdc::errors::Error;
use mysql_cdc::events::binlog_event::BinlogEvent;
use mysql_cdc::replica_options::ReplicaOptions;
use mysql_cdc::ssl_mode::SslMode;

pub fn run_slave() -> Result<(), Error> {
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
                log::trace!("write event")
            }
            _ => {
                log::trace!("ignore event")
            }
        }

        // You process an event here

        // After you processed the event, you need to update replication position
    }
    Ok(())
}
