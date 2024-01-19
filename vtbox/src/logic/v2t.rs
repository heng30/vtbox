use crate::slint_generatedAppWindow::{AppWindow, Logic, Store};
use crate::util::translator::tr;
use crate::{
    config,
    message::async_message_warn,
    model::model_relative_path,
    transcribe::{model_handler, transcriber},
};
use crate::{message_info, message_success, message_warn};
use anyhow::Result;
use native_dialog::FileDialog;
use slint::{ComponentHandle, Model, SharedString};
use tokio::task::spawn;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_CONVERTING: AtomicBool = AtomicBool::new(false);

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>().on_open_audio_file(move || {
        let ui = ui_handle.unwrap();

        match FileDialog::new()
            .set_location("~")
            .add_filter("Audio", &["mp3", "wav"])
            .show_open_single_file()
        {
            Ok(Some(file)) => file.to_str().unwrap().into(),
            Err(e) => {
                message_warn!(
                    &ui,
                    format!("{}. {}: {e:?}", tr("打开文件失败"), tr("原因"))
                );
                SharedString::default()
            }
            _ => SharedString::default(),
        }
    });

    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_start_v2t(move |model_name, audio_path| {
            let ui = ui_handle.unwrap();

            if IS_CONVERTING.load(Ordering::SeqCst) {
                message_warn!(ui, tr("有任务正在运行，稍后重试"));
                return;
            }

            if ui.global::<Store>().get_v2t_models().row_count() == 0 {
                message_info!(ui, tr("请先下载模型"));
                return;
            }

            if model_name.is_empty() || audio_path.is_empty() {
                message_warn!(ui, tr("非法输入"));
                return;
            }

            message_info!(ui, tr("正在转换..."));

            let (ui, model_name, audio_path) =
                (ui.as_weak(), model_name.to_string(), audio_path.to_string());

            spawn(async move {
                IS_CONVERTING.store(true, Ordering::SeqCst);

                match inner_start_v2t(&model_name, &audio_path) {
                    Err(e) => async_message_warn(
                        ui.clone(),
                        format!("{}. {}: {e:?}", tr("转换失败"), tr("原因")),
                    ),
                    Ok(text) => {
                        let _ = slint::invoke_from_event_loop(move || {
                            let ui = ui.clone().unwrap();
                            ui.global::<Store>().set_v2t_text(text.into());
                            message_success!(&ui, tr("转换成功"));
                        });
                    }
                }

                IS_CONVERTING.store(false, Ordering::SeqCst);
            });
        });
}

fn inner_start_v2t(model_name: &str, audio_path: &str) -> Result<String> {
    let models_dir = format!("{}/{}", config::cache_dir(), model_relative_path(0));

    let m = model_handler::ModelHandler::new(model_name, &models_dir)?;
    let trans = transcriber::Transcriber::new(m)?;
    let result = trans.transcribe(audio_path, None)?;
    Ok(result.get_text().to_string())
}
