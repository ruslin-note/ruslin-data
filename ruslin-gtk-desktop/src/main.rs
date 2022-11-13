mod ui;

use gtk::prelude::*;
use gtk::{gio, Application};
use ui::main_window::MainWindow;

const APP_ID: &str = "org.dianqk.ruslin-desktop.gtk";

fn main() {
    // Register and include resources
    gio::resources_register_include!("ruslin.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}
fn build_ui(app: &Application) {
    // Create new window and present it
    let window = MainWindow::new(app);
    window.present();
}
