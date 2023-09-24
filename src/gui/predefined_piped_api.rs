/*
 * Copyright 2021 - 2022 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Pipeline.
 *
 * Pipeline is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Pipeline is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Pipeline.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

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
