// Evita ventanas de consola adicionales en Windows en la versión,¡NO ELIMINAR!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    yarvis_app_lib::run()
}
