use crate::task::SearchTask;
use std::collections::HashMap;

pub fn handle(task: SearchTask) {
    let url = task.http_task.raw_req.url();
    let quest_pos = url.find("?").unwrap_or(url.len() - 1);
    let params_str = url[quest_pos + 1..url.len()].to_string();

    let mut params = HashMap::new();
    for pair in params_str.split('&') {
        let mut split = pair.split('=');
        let key = split.next().unwrap_or("");
        let value = split.next().unwrap_or("");
        params.insert(key.to_string(), value.to_string());
    }
    task.http_task.respond_html(
        format!(
            "search_task: {} = {}",
            "req",
            params.get("req").unwrap_or(&"not found".to_string())
        )
        .as_str(),
    );
}
