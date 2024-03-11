use crate::task::SearchTask;

pub fn handle(task: SearchTask) {
    match task.search_request {
        Ok(search_req) => {
            let id = search_req.search_params.id;
            let mut response = format!("search_params = {:?}", search_req);

            match task
                .context
                .store
                .get_raw_data()
                .try_get::<crate::data::objects::Campaign>(id)
            {
                Some(campaign) => {
                    response += format!("</br>campaign found: {:?}", campaign).as_str();
                }
                None => {
                    response += String::from("</br>campaign not found").as_str();
                }
            };

            task.http_task.respond_html(response.as_str());
        }
        Err(e) => {
            task.http_task
                .respond_html(format!("malformed request: {}", e).as_str());
            log::warn!("malformed request: {}\n{}", e, e.backtrace());
        }
    }
}
