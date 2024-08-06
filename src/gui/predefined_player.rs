use gtk::glib::Object;

gtk::glib::wrapper! {
    pub struct PredefinedPlayer(ObjectSubclass<imp::PredefinedPlayer>);
}

impl PredefinedPlayer {
    pub fn new<S: AsRef<str>>(name: S, command: S) -> Self {
        Object::builder()
            .property("name", name.as_ref())
            .property("command", command.as_ref())
            .build()
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk_pixbuf::glib::Properties;

    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::PredefinedPlayer)]
    pub struct PredefinedPlayer {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        command: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PredefinedPlayer {
        const NAME: &'static str = "TFPredefinedPlayer";
        type Type = super::PredefinedPlayer;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PredefinedPlayer {}
}
