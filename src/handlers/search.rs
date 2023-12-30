use crate::task::SearchTask;

pub fn handle(task: SearchTask) {
    match task.is_malformed {
        true => {
            task.http_task
                .respond_html(format!("malformed request: {}", task.malformed_msg).as_str());
        }
        false => {
            task.http_task
                .respond_html(format!("search_params = {:?}", task.search_params).as_str());
        }
    }
}
