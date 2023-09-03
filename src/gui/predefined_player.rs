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
