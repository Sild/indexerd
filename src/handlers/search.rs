use crate::task::SearchTask;

pub fn handle(task: SearchTask) {
    task.http_task.respond("admin_task");
}
