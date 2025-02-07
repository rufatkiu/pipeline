use std::cell::RefCell;

use gdk::glib::Object;
use gdk::subclass::prelude::ObjectSubclassIsExt;
use tf_join::Platform;

macro_rules! str_prop {
    ( $x:expr ) => {
        ParamSpecString::builder($x).build()
    };
}

macro_rules! prop_set {
    ( $x:expr, $value:expr ) => {
        let input = $value
            .get::<'_, Option<String>>()
            .expect("The value needs to be of type `Option<String>`.");
        $x.replace(input);
    };
}

macro_rules! prop_set_all {
    ( $value:expr, $pspec:expr, $( $key:expr, $element:expr ),* ) => {
        match $pspec.name() {
            $(
                $key => { prop_set!($element, $value); },
            )*
                _ => unimplemented!()
        }
    }
}

macro_rules! prop_get_all {
    ( $pspec:expr, $( $key:expr, $element:expr ),* ) => {
        match $pspec.name() {
            $(
                $key => { $element.borrow().to_value() },
            )*
                _ => unimplemented!()
        }
    }
}

gtk::glib::wrapper! {
    pub struct PlatformObject(ObjectSubclass<imp::PlatformObject>);
}

impl PlatformObject {
    pub fn new(platform: Platform) -> Self {
        let s: Self = Object::builder::<Self>()
            .property("name", platform.to_string())
            .build();
        s.imp().platform.swap(&RefCell::new(Some(platform)));
        s
    }

    pub fn platform(&self) -> Option<Platform> {
        self.imp().platform.borrow().clone()
    }
}

mod imp {
    use gtk::glib;
    use std::cell::RefCell;
    use tf_join::Platform;

    use gdk::{
        glib::{ParamSpec, ParamSpecString, Value},
        prelude::ToValue,
        subclass::prelude::{ObjectImpl, ObjectSubclass},
    };
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct PlatformObject {
        name: RefCell<Option<String>>,

        pub(super) platform: RefCell<Option<Platform>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlatformObject {
        const NAME: &'static str = "TFPlatformObject";
        type Type = super::PlatformObject;
    }

    impl ObjectImpl for PlatformObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![str_prop!("name")]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            prop_set_all!(value, pspec, "name", self.name);
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            prop_get_all!(pspec, "name", self.name)
        }
    }
}
