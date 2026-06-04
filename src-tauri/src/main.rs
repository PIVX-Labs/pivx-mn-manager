// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ssh;

fn main() {
    pivx_mn_manager_lib::run()
}
