use crate::data::objects::{Campaign, IdType, Package, Pad};
use crate::data::objects_traits::MysqlObject;
use crate::data::slave;
use crate::data::store::IndexStat;
use crate::task::AdminTask;
use serde::{Deserialize, Serialize};
use tera::{Error, Tera};

// static my_str: &str = include_str!("$CARGO_MANIFEST_DIR/src/html_tpl/admin.html");

pub fn handle(task: AdminTask) {
    log::debug!("start handing admin task: {}", task.http_task.url());
    let url = task.http_task.url();
    let _ = url.strip_suffix('/');
    let html_response = {
        if url.eq("/admin") {
            handle_root(&task)
        } else if url.eq("/admin/status") {
            handle_status(&task)
        } else if url.eq("/admin/store") {
            handle_store(&task)
        } else if url.starts_with("/admin/store/") {
            let paths = url.split('/').collect::<Vec<&str>>();
            if paths.len() == 4 {
                // /admin/store/campaign
                handle_objects(&task, paths[3])
            } else {
                // /admin/store/campaign/1
                handle_object_detailed(&task, paths[3], paths[4].parse::<i32>().unwrap())
            }
        } else {
            Ok(format!("unknown url: {}", url))
        }
    };

    match html_response {
        Ok(rsp) => task.http_task.respond_html(&rsp),
        Err(e) => task
            .http_task
            .respond_html(&format!("handling error: {:?}", e)),
    }
}

fn handle_root(_: &AdminTask) -> Result<String, Error> {
    let tpl_name = "admin.html";
    let tpl_data = include_str!("../html_tpl/admin.html");
    let mut tera = Tera::default();
    tera.add_raw_template(tpl_name, tpl_data)?;
    let context = tera::Context::new();
    tera.render(tpl_name, &context)
}

fn handle_status(task: &AdminTask) -> Result<String, Error> {
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
    Ok(serde_json::to_string(&status).unwrap_or("fail to deserialize".to_string()))
}

fn handle_store(_task: &AdminTask) -> Result<String, Error> {
    let tpl_name = "tpl";
    let tpl_data = include_str!("../html_tpl/admin_store.html");
    let mut tera = Tera::default();
    tera.add_raw_template(tpl_name, tpl_data)?;
    let mut context = tera::Context::new();
    context.insert(
        "objects",
        &vec![Campaign::table(), Package::table(), Pad::table()],
    );

    tera.render(tpl_name, &context)
}

fn handle_objects(task: &AdminTask, object_type: &str) -> Result<String, Error> {
    let store_row_data = task.context.store.get_raw_data();
    let mut objects = match slave::SupportedTypes::from(object_type) {
        slave::SupportedTypes::Campaign => store_row_data.list::<Campaign>(),
        slave::SupportedTypes::Package => store_row_data.list::<Package>(),
        slave::SupportedTypes::Pad => store_row_data.list::<Pad>(),
        _ => return Err("unknown object type".into()),
    };
    objects.sort();

    let tpl_name = "tpl";
    let tpl_data = include_str!("../html_tpl/admin_store_objects.html");
    let mut tera = Tera::default();
    tera.add_raw_template(tpl_name, tpl_data)?;
    let mut context = tera::Context::new();
    context.insert("object_type", object_type);
    context.insert("objects", &objects);

    tera.render(tpl_name, &context)
}

fn handle_object_detailed(
    task: &AdminTask,
    object_type: &str,
    object_id: IdType,
) -> Result<String, Error> {
    let store_rd = task.context.store.get_raw_data();

    let tpl_name = "tpl";
    let tpl_data = include_str!("../html_tpl/admin_store_objects_detailed.html");
    let mut tera = Tera::default();
    tera.add_raw_template(tpl_name, tpl_data)?;
    let mut context = tera::Context::new();
    context.insert("object_type", object_type);

    match slave::SupportedTypes::from(object_type) {
        slave::SupportedTypes::Campaign => match store_rd.try_get::<Campaign>(object_id) {
            Some(campaign) => context.insert("object", campaign.html_debug().as_str()),
            None => context.insert("object", "campaign not found"),
        },
        slave::SupportedTypes::Package => context.insert(
            "object",
            format!("{:?}", store_rd.try_get::<Package>(object_id)).as_str(),
        ),
        slave::SupportedTypes::Pad => context.insert(
            "object",
            format!("{:?}", store_rd.try_get::<Pad>(object_id)).as_str(),
        ),
        _ => return Err("unknown object type".into()),
    };
    context.insert("object_id", &object_id);
    tera.render(tpl_name, &context)
}
