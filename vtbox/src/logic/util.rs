use crate::slint_generatedAppWindow::{AppWindow, Util};
use crate::util::{self, number, time};
use slint::ComponentHandle;

pub fn init(ui: &AppWindow) {
    ui.global::<Util>().on_string_fixed2(move |n| {
        let n = n.to_string().parse::<f32>().unwrap_or(0.0f32);
        slint::format!("{:2}", (n * 100.0).round() / 100.0)
    });

    ui.global::<Util>()
        .on_float_fixed2(move |n| slint::format!("{:2}", (n * 100.0).round() / 100.0));

    ui.global::<Util>()
        .on_format_number_with_commas(move |number_str| {
            number::format_number_with_commas(number_str.as_str()).into()
        });

    ui.global::<Util>()
        .on_local_now(move |format| time::local_now(format.as_str()).into());

    ui.global::<Util>()
        .on_split_and_join_string(move |input, length, sep| {
            util::str::split_string_to_fixed_length_parts(input.as_str(), length as usize)
                .join(sep.as_str())
                .into()
        });

    ui.global::<Util>()
        .on_filename(move |path| util::fs::filename(&path).unwrap_or("".to_string()).into());
}
