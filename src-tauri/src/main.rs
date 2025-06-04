// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
use commands::analyze_audio;

fn main() {
    beatbeatrebellion_lib::run()
}
