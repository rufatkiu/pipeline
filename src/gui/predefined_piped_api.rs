use gtk::glib::Object;

gtk::glib::wrapper! {
    pub struct PredefinedPipedApi(ObjectSubclass<imp::PredefinedPipedApi>);
}

impl PredefinedPipedApi {
    pub fn new<S: AsRef<str>>(name: S, url: S) -> Self {
        Object::builder()
            .property("name", name.as_ref())
            .property("url", url.as_ref())
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
    #[properties(wrapper_type = super::PredefinedPipedApi)]
    pub struct PredefinedPipedApi {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        url: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PredefinedPipedApi {
        const NAME: &'static str = "TFPredefinedPipedApi";
        type Type = super::PredefinedPipedApi;
    }

    #[glib::derived_properties]
    impl ObjectImpl for PredefinedPipedApi {}
}
