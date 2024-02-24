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

use gdk_pixbuf::{
    glib,
    prelude::IsA,
    subclass::prelude::{IsSubclassable, IsSubclassableExt},
};
use adw::subclass::prelude::BinImpl;

gtk::glib::wrapper! {
    pub struct StackPage(ObjectSubclass<imp::StackPage>)
        @extends adw::Bin, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

pub trait StackPageImpl: BinImpl {}

unsafe impl<T> IsSubclassable<T> for StackPage
where
    T: StackPageImpl,
    T::Type: IsA<StackPage>,
{
    fn class_init(class: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(class.upcast_ref_mut());
    }
}

pub mod imp {
    use std::cell::RefCell;

    use gdk_pixbuf::glib::Properties;
    use gtk::glib;
    use gtk::Widget;

    use gtk::prelude::ObjectExt;
    use gtk::subclass::prelude::*;

    use adw::subclass::prelude::BinImpl;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::StackPage)]
    pub struct StackPage {
        #[property(get, set, nullable)]
        header_widget: RefCell<Option<Widget>>,
        #[property(get, set)]
        name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StackPage {
        const NAME: &'static str = "TFStackPage";
        type Type = super::StackPage;
        type ParentType = adw::Bin;
    }

    #[glib::derived_properties]
    impl ObjectImpl for StackPage {}

    impl WidgetImpl for StackPage {}
    impl BinImpl for StackPage {}
}
