mod app;
mod pages;

use crate::app::App;

fn main() {
    leptos::mount::mount_to_body(App);
}
