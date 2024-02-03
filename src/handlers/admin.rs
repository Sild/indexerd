use crate::data::objects::{Campaign, Package, Pad};
use crate::data::objects_traits::MysqlObject;
use crate::data::store::IndexStat;
use crate::task::AdminTask;
use serde::{Deserialize, Serialize};
use tera::{Error, Tera};

// static my_str: &str = include_str!("$CARGO_MANIFEST_DIR/src/html_tpl/admin.html");

pub fn handle(task: AdminTask) {
    let url = task.http_task.url();
    let _ = url.strip_suffix('/');
    let html_response = match url {
        "/admin" => handle_root(&task),
        "/admin/status" => handle_status(&task),
        "/admin/store/campaign" => handle_objects(&task),
        "/admin/store" => handle_store(&task),
        _ => Ok(format!("unknown url: {}", url)),
    };
    match html_response {
        Ok(rsp) => task.http_task.respond_html(&rsp),
        Err(e) => task.http_task.respond_html(&format!("error: {}", e)),
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
    let tpl_name = "admin_store.html";
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

fn handle_objects(task: &AdminTask) -> Result<String, Error> {
    let tpl_name = "admin_store.html";
    let tpl_data = include_str!("../html_tpl/admin_store_objects.html");
    let mut tera = Tera::default();
    tera.add_raw_template(tpl_name, tpl_data)?;
    let mut context = tera::Context::new();
    context.insert("object_type", "campaign");
    let mut objects = task.context.store.get_raw_data().list::<Campaign>();
    objects.sort();
    context.insert("objects", &objects);

    tera.render(tpl_name, &context)
}
