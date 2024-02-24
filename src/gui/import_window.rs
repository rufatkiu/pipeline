use gdk_pixbuf::glib::clone;
use gtk::glib;
use gtk::prelude::*;
use gtk::Builder;
use gtk::FileDialog;
use gtk::FileFilter;
use adw::prelude::MessageDialogExt;
use adw::MessageDialog;
use tf_join::Joiner;

pub fn import_window(joiner: Joiner, parent: &crate::gui::window::Window) -> MessageDialog {
    let builder = Builder::from_resource("/ui/import_window.ui");
    let dialog: MessageDialog = builder
        .object("dialog")
        .expect("import_window.ui to have at least one object dialog");
    dialog.set_transient_for(Some(parent));
    dialog.set_modal(true);
    dialog.connect_response(
        None,
        clone!(@strong joiner, @weak parent => move |_dialog, response| {
            handle_response(&joiner, response, &parent);
        }),
    );
    dialog
}

fn handle_response(joiner: &Joiner, response: &str, parent: &crate::gui::window::Window) {
    match response {
        "newpipe" => {
            log::debug!("Import from NewPipe");
            let filter = FileFilter::new();
            filter.add_mime_type("application/json");
            let chooser = FileDialog::builder()
                .title(&gettextrs::gettext("Select NewPipe subscriptions file"))
                .modal(true)
                .default_filter(&filter)
                .build();
            chooser.open(
                Some(parent),
                None::<&gtk::gio::Cancellable>,
                clone!(@strong chooser, @strong joiner => move |file| {
                    if let Ok(file) = file {
                        log::trace!("User picked file to import from");
                        if let Err(e) = crate::import::import_newpipe(&joiner, file) {
                            let dialog = MessageDialog::builder()
                                .heading(&gettextrs::gettext("Failure to import subscriptions"))
                                .body(&format!("{}", e))
                                .build();
                            dialog.present();
                        }
                    } else {
                        log::trace!("User did not choose anything to import from");
                    }
                }),
            );
        }
        "youtube" => {
            log::debug!("Import from YouTube");
            let filter = FileFilter::new();
            filter.add_mime_type("text/csv");
            let chooser = FileDialog::builder()
                .title(&gettextrs::gettext("Select YouTube subscription file"))
                .modal(true)
                .default_filter(&filter)
                .build();
            chooser.open(
                Some(parent),
                None::<&gtk::gio::Cancellable>,
                clone!(@strong chooser, @strong joiner => move |file| {
                    if let Ok(file) = file {
                        log::trace!("User picked file to import from");
                        if let Err(e) = crate::import::import_youtube(&joiner, file) {
                            let dialog = MessageDialog::builder()
                                .heading(&gettextrs::gettext("Failure to import subscriptions"))
                                .body(&format!("{}", e))
                                .build();
                            dialog.present();
                        }
                    } else {
                        log::trace!("User did not choose anything to import from");
                    }
                }),
            );
        }
        _ => {}
    }
}
