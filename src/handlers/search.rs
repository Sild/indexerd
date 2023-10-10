use crate::task::SearchTask;

pub fn handle(task: SearchTask) {
    task.http_task.respond_html("search_task");
}
