use gdk::{
    prelude::{ApplicationExt, ApplicationExtManual},
    Display,
};
use gdk_pixbuf::{gio::Settings, prelude::SettingsExt};
use gtk::{prelude::GtkWindowExt, CssProvider};

mod config;
use self::config::{APP_ID, GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_BYTES};

mod csv_file_manager;
mod downloader;
mod gui;
mod import;
mod player;

#[macro_export]
macro_rules! gspawn {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn_local($future);
    };
}
#[macro_export]
macro_rules! gspawn_global {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn($future);
    };
}

fn init_setting(env: &'static str, value: &str) {
    if std::env::var_os(env).is_none() {
        std::env::set_var(env, value);
    }
}

fn init_settings() {
    let settings = Settings::new(APP_ID);
    init_setting("PLAYER", &settings.string("player"));
    init_setting("DOWNLOADER", &settings.string("downloader"));
    init_setting("PIPED_API_URL", &settings.string("piped-url"));
}

fn init_resources() {
    let gbytes = gtk::glib::Bytes::from_static(RESOURCES_BYTES);
    let resource = gtk::gio::Resource::from_data(&gbytes).unwrap();

    gtk::gio::resources_register(&resource);
}

fn init_folders() {
    let mut user_cache_dir = gtk::glib::user_cache_dir();
    user_cache_dir.push("tubefeeder");

    if !user_cache_dir.exists() {
        std::fs::create_dir_all(&user_cache_dir).expect("could not create user cache dir");
    }

    let mut user_data_dir = gtk::glib::user_data_dir();
    user_data_dir.push("tubefeeder");

    if !user_data_dir.exists() {
        std::fs::create_dir_all(user_data_dir.clone()).expect("could not create user data dir");
    }
}

fn init_internationalization() -> Result<(), Box<dyn std::error::Error>> {
    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)?;
    gettextrs::textdomain(GETTEXT_PACKAGE)?;
    Ok(())
}

fn init_css() {
    let provider = CssProvider::new();
    provider.load_from_resource("/de/schmidhuberj/tubefeeder/style.css");

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[tokio::main]
async fn main() {
    env_logger::init();
    init_internationalization().expect("Failed to initialize internationalization");

    gtk::init().expect("Failed to initialize gtk");
    adw::init().expect("Failed to initialize adw");
    let app = gtk::Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &gtk::Application) {
    init_resources();
    init_folders();
    init_settings();
    init_css();
    // Create new window and present it
    let window = crate::gui::window::Window::new(app);
    window.present();
}
