use crate::task::SearchTask;

pub fn handle(task: SearchTask) {
    match task.search_request {
        Ok(search_req) => {
            task.http_task
                .respond_html(format!("search_params = {:?}", search_req).as_str());
        }
        Err(e) => {
            task.http_task.respond_html(
                format!("malformed request: {}\n{}", e.to_string(), e.backtrace()).as_str(),
            );
        }
    }
}
