use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

const APP_ID: &str = "org.dianqk.ruslin-desktop.gtk";

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Ruslin")
        .build();

    // Present window
    window.present();
}
