#![windows_subsystem = "windows"]

slint::include_modules!();

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lazy_static;

mod config;
mod logic;
mod util;
mod version;

use logic::{about, clipboard, message, model, ok_cancel_dialog, setting, v2t, window};

use anyhow::Result;
use chrono::Local;
use env_logger::fmt::Color as LColor;
use log::debug;
use std::io::Write;
use transcribe;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();
    debug!("start...");

    config::init();

    let ui = AppWindow::new()?;

    logic::util::init(&ui);
    logic::base::init(&ui);

    clipboard::init(&ui);
    message::init(&ui);
    window::init(&ui);
    about::init(&ui);
    setting::init(&ui);
    ok_cancel_dialog::init(&ui);

    model::init(&ui);
    v2t::init(&ui);

    ui.run().unwrap();

    debug!("exit...");
    Ok(())
}

fn init_logger() {
    env_logger::builder()
        .format(|buf, record| {
            let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
            let mut level_style = buf.style();
            match record.level() {
                log::Level::Warn | log::Level::Error => {
                    level_style.set_color(LColor::Red).set_bold(true)
                }
                _ => level_style.set_color(LColor::Blue).set_bold(true),
            };

            writeln!(
                buf,
                "[{} {} {} {}] {}",
                ts,
                level_style.value(record.level()),
                record
                    .file()
                    .unwrap_or("None")
                    .split('/')
                    .last()
                    .unwrap_or("None"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
}
