use crate::data::store::IndexStat;
use crate::task::AdminTask;
use serde::{Deserialize, Serialize};

pub fn handle(task: AdminTask) {
    if task.http_task.raw_req.url().starts_with("/admin/status") {
        handle_status(task);
    } else if task.http_task.raw_req.url().starts_with("/admin/store") {
        handle_debug(task);
    }
}

fn handle_status(task: AdminTask) {
    #[derive(Deserialize, Serialize)]
    struct Status {
        is_ready: bool,
        index_stat: IndexStat,
    }

    let mut status = Status {
        is_ready: false,
        index_stat: task.context.store.get_store_stat().clone(),
    };
    status.is_ready = status.index_stat.iteration != 0;
    let response = serde_json::to_string(&status).unwrap_or("fail to deserialize".to_string());
    task.http_task.respond(&response);
}

fn handle_debug(task: AdminTask) {
    task.http_task.respond("/admin/store");
}
