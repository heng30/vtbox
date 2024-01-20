use crate::slint_generatedAppWindow::{AppWindow, Logic, ModelItem, Store};
use crate::util::translator::tr;
use crate::{config, message::async_message_warn, util};
use crate::{message_info, message_success, message_warn};
use anyhow::Result;
use native_dialog::FileDialog;
use slint::{ComponentHandle, Model, SharedString, VecModel};
use std::fs;
use tokio::task::spawn;
use transcribe::model_handler;
use uuid::Uuid;

const PREDEFINED_MODELS_V2T: [&str; 5] = [
    "ggml-tiny.bin",
    "ggml-base.bin",
    "ggml-small.bin",
    "ggml-medium.bin",
    "ggml-large.bin",
];

pub fn init(ui: &AppWindow) {
    init_model(ui, 0);

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_remove_model(move |type_index, uuid| {
            if uuid.is_empty() {
                return;
            }

            let ui = ui_handle.unwrap();

            for item in ui.global::<Store>().get_model_datas().iter() {
                if item.uuid != uuid {
                    continue;
                }

                if item.status == "Undownload" {
                    message_warn!(&ui, tr("无法删除，文件不存在"));
                    break;
                }

                let file = format!(
                    "{}/{}/{}",
                    config::cache_dir(),
                    model_relative_path(type_index),
                    item.name
                );
                let _ = fs::remove_file(file);

                init_model(&ui, type_index);
                message_success!(ui, tr("删除成功"));
                break;
            }
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_refresh_model(move |type_index| {
        let ui = ui_handle.unwrap();

        init_model(&ui, type_index);
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_switch_model_type(move |type_index| {
            let ui = ui_handle.unwrap();
            init_model(&ui, type_index);
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_download_model(move |type_index, uuid| {
            if uuid.is_empty() {
                return;
            }

            let ui = ui_handle.unwrap();

            let model = match get_model_data(&ui, &uuid) {
                Some(v) => v,
                _ => {
                    message_warn!(&ui, tr("内部错误，请刷新列表"));
                    return;
                }
            };
            let name = model.name;

            if !is_in_predefined_models(type_index, &name) {
                message_info!(&ui, tr("不支持下载"));
                return;
            }

            message_info!(&ui, tr("正在下载..."));

            let (ui, name) = (ui.as_weak(), name.to_string());
            spawn(async move {
                match inner_download_model(type_index, &name).await {
                    Err(e) => async_message_warn(
                        ui.clone(),
                        format!("{}. {}: {e:?}", tr("下载失败"), tr("原因")),
                    ),
                    _ => {
                        let _ = slint::invoke_from_event_loop(move || {
                            let ui = ui.clone().unwrap();
                            if type_index == ui.get_model_type_index() {
                                init_model(&ui, type_index);
                            }

                            message_success!(ui, tr("下载成功"));
                        });
                    }
                }
            });
        });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_import_model(move |type_index| {
        let ui = ui_handle.unwrap();

        match FileDialog::new().set_location("~").show_open_single_file() {
            Ok(Some(file)) => {
                let path = format!(
                    "{}/{}/{}",
                    config::cache_dir(),
                    model_relative_path(type_index),
                    file.file_name().unwrap().to_str().unwrap(),
                );

                match fs::copy(file, path) {
                    Err(e) => {
                        message_warn!(&ui, format!("{}. {}: {e:?}", tr("导入失败"), tr("原因")));
                    }
                    _ => {
                        init_model(&ui, type_index);
                        message_success!(&ui, tr("导入成功"));
                    }
                }
            }
            Err(e) => {
                message_warn!(&ui, format!("{}. {}: {e:?}", tr("导入失败"), tr("原因")));
            }

            _ => (),
        };
    });
}

fn init_model(ui: &AppWindow, type_index: i32) {
    let cache_dir = config::cache_dir();
    let _ = std::fs::create_dir_all(format!("{}/{}", cache_dir, model_relative_path(0)));
    let _ = std::fs::create_dir_all(format!("{}/{}", cache_dir, model_relative_path(1)));

    let items = match model_items(ui, type_index) {
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
}

pub fn model_relative_path(type_index: i32) -> String {
    if type_index == 0 {
        "v2t/model".to_string()
    } else {
        "t2v/model".to_string()
    }
}

async fn inner_download_model(type_index: i32, name: &str) -> Result<()> {
    let proxy_config = config::socks5();
    let path = format!(
        "{}/{}",
        config::cache_dir(),
        model_relative_path(type_index)
    );
    let proxy_info = if proxy_config.enabled {
        Some((proxy_config.url.as_str(), proxy_config.port))
    } else {
        None
    };

    model_handler::download_model(&path, name, proxy_info).await
}

fn model_items(ui: &AppWindow, type_index: i32) -> Result<Vec<ModelItem>> {
    let path = format!(
        "{}/{}",
        config::cache_dir(),
        model_relative_path(type_index)
    );

    let mut models: Vec<_> = fs::read_dir(path)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                let name = path.file_name()?.to_str()?;
                let size = util::fs::pretty_size(entry.metadata().ok()?.len());
                let status = if is_in_predefined_models(type_index, name) {
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
        .collect();

    set_combobox_models(ui, type_index, &models);
    append_undownload_model(type_index, &mut models);
    Ok(models)
}

fn is_in_predefined_models(type_index: i32, name: &str) -> bool {
    if type_index == 0 {
        for item in PREDEFINED_MODELS_V2T {
            if item == name {
                return true;
            }
        }
    }

    false
}

fn is_in_models(items: &[ModelItem], name: &str) -> bool {
    items.iter().any(|item| item.name == name)
}

fn get_model_data(ui: &AppWindow, uuid: &str) -> Option<ModelItem> {
    ui.global::<Store>()
        .get_model_datas()
        .iter()
        .find(|item| item.uuid == uuid)
}

fn append_undownload_model(type_index: i32, models: &mut Vec<ModelItem>) {
    let mut tmp_items = vec![];
    if type_index == 0 {
        for name in PREDEFINED_MODELS_V2T {
            if !is_in_models(models, name) {
                tmp_items.push(ModelItem {
                    uuid: Uuid::new_v4().to_string().into(),
                    name: name.into(),
                    size: "-".into(),
                    status: "Undownload".into(),
                });
            }
        }
    }
    models.append(&mut tmp_items);
}

fn set_combobox_models(ui: &AppWindow, type_index: i32, models: &[ModelItem]) {
    let items = models
        .iter()
        .map(|item| item.name.clone())
        .collect::<Vec<_>>();

    if type_index == 0 {
        ui.global::<Store>()
            .get_v2t_models()
            .as_any()
            .downcast_ref::<VecModel<SharedString>>()
            .expect("We know we set a VecModel earlier")
            .set_vec(items);
    }
}
