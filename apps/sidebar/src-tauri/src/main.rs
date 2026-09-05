// Verhindert ein zusätzliches Konsolenfenster im Release-Build unter Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    todosidebar_lib::run()
}
