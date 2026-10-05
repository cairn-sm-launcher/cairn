// Sans cet attribut, une fenetre de console noire s'ouvre derriere
// l'application dans la version publiee. En developpement on la garde,
// c'est la ou atterrissent les messages.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    cairn_app::run()
}
