use crate::slint_generatedAppWindow::{AppWindow, Logic};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.global::<Logic>()
        .on_handle_ok_cancel_dialog(move |handle_type, handle_uuid| {
            let ui = ui_handle.unwrap();

            match handle_type.as_str() {
                "remove-model" => {
                    let items: Vec<&str> = handle_uuid.split(',').collect();
                    ui.global::<Logic>()
                        .invoke_remove_model(items[0].parse::<_>().unwrap(), items[1].into());
                }
                _ => (),
            }
        });
}
