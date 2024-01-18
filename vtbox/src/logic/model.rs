use crate::slint_generatedAppWindow::{AppWindow, Logic, ModelItem, Store};
use crate::util::translator::tr;
use crate::{config, util};
use crate::{message_success, message_warn};
use anyhow::Result;
use slint::{ComponentHandle, Model, VecModel, Weak};
use std::fs;
use uuid::Uuid;

const PREDEFINED_MODELS: [&str; 5] = [
    "ggml-tiny.bin",
    "ggml-base.bin",
    "ggml-small.bin",
    "ggml-medium.bin",
    "ggml-large.bin",
];

pub fn init(ui: &AppWindow) {
    let items = match model_items(ui, 0) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("get model items error: {e:?}");
            vec![]
        }
    };
    ui.global::<Store>()
        .get_model_datas()
        .as_any()
        .downcast_ref::<VecModel<ModelItem>>()
        .expect("We know we set a VecModel earlier")
        .set_vec(items);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_remove_model(move |type_index, uuid| {
            let ui = ui_handle.unwrap();

            for (index, item) in ui.global::<Store>().get_model_datas().iter().enumerate() {
                if item.uuid != uuid {
                    continue;
                }

                ui.global::<Store>()
                    .get_model_datas()
                    .as_any()
                    .downcast_ref::<VecModel<ModelItem>>()
                    .expect("We know we set a VecModel earlier")
                    .remove(index);

                let file = format!(
                    "{}/{}/{}",
                    config::cache_dir(),
                    model_type(type_index),
                    item.name
                );
                let _ = fs::remove_file(file);
                break;
            }

            message_success!(ui, tr("删除成功"));
        });
}

fn model_type(type_index: i32) -> String {
    if type_index == 0 {
        "v2t/model".to_string()
    } else {
        "t2v/model".to_string()
    }
}

fn model_items(ui: &AppWindow, type_index: i32) -> Result<Vec<ModelItem>> {
    let path = format!("{}/{}", config::cache_dir(), model_type(type_index));

    Ok(fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name()?.to_str()?;
                let size = util::fs::pretty_size(entry.metadata().ok()?.len());
                let status = if is_in_predefined_models(name) {
                    "Downloaded"
                } else {
                    "Imported"
                };

                Some(ModelItem {
                    uuid: Uuid::new_v4().to_string().into(),
                    name: name.into(),
                    size: size.into(),
                    status: status.into(),
                })
            } else {
                None
            }
        })
        .collect())
}

fn is_in_predefined_models(name: &str) -> bool {
    for item in PREDEFINED_MODELS.iter() {
        if *item == name {
            return true;
        }
    }

    false
}
