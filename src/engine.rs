use crate::store::data_manager::DataManager;
// use crate::server::Server;

pub struct Engine {
    data_manager: DataManager,
    // web_server: Server,
}

impl Engine {
    fn shutdown() {
        println!("going to shutdown");
    }
}