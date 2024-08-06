use std::sync::{Arc, Mutex};

use gdk::subclass::prelude::ObjectSubclassIsExt;
use gdk_pixbuf::prelude::Cast;
use gtk::prelude::WidgetExt;
use tf_filter::FilterGroup;
use tf_join::AnyVideoFilter;

use crate::gui::stack_page::StackPage;

gtk::glib::wrapper! {
    pub struct FilterPage(ObjectSubclass<imp::FilterPage>)
        @extends StackPage, adw::Bin, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FilterPage {
    pub fn set_filter_group(&self, filter_group: Arc<Mutex<FilterGroup<AnyVideoFilter>>>) {
        self.imp().filter_group.replace(Some(filter_group.clone()));
        self.imp().filter_list.get().set_filter_group(filter_group);
    }

    fn window(&self) -> crate::gui::window::Window {
        self.root()
            .expect("FilterPage to have root")
            .downcast::<crate::gui::window::Window>()
            .expect("Root to be window")
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use adw::prelude::AdwDialogExt;
    use gdk::glib::clone;
    use gdk::glib::ParamSpec;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    use adw::subclass::prelude::BinImpl;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use regex::Regex;
    use tf_filter::FilterGroup;
    use tf_join::AnyVideoFilter;

    use crate::gui::filter::filter_list::FilterList;
    use crate::gui::stack_page::StackPage;
    use crate::gui::stack_page::StackPageImpl;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/filter_page.ui")]
    pub struct FilterPage {
        #[template_child]
        pub(super) filter_list: TemplateChild<FilterList>,

        #[template_child]
        pub(super) btn_toggle_add_filter: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) btn_add_filter: TemplateChild<gtk::Button>,
        #[template_child]
        pub(super) entry_title: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) entry_channel: TemplateChild<gtk::Entry>,
        #[template_child]
        pub(super) dialog_add: TemplateChild<adw::AlertDialog>,

        pub(super) filter_group: RefCell<Option<Arc<Mutex<FilterGroup<AnyVideoFilter>>>>>,
    }

    #[gtk::template_callbacks]
    impl FilterPage {
        fn present_filter(&self) {
            self.entry_channel.set_text("");
            self.entry_title.set_text("");

            let window = self.obj().window();
            self.dialog_add.present(Some(&window));
        }

        fn setup_toggle_add_filter(&self, obj: &super::FilterPage) {
            self.btn_toggle_add_filter.connect_clicked(clone!(
                #[strong]
                obj,
                move |_| {
                    obj.imp().present_filter();
                }
            ));
            self.btn_add_filter.connect_clicked(clone!(
                #[strong]
                obj,
                move |_| {
                    obj.imp().present_filter();
                }
            ));
        }

        #[template_callback]
        fn handle_add_filter(&self, response: Option<&str>) {
            if response != Some("add") {
                return;
            }

            let in_title = &self.entry_title;
            let in_channel = &self.entry_channel;
            let filters = &self.filter_group;

            let title = in_title.text();
            let channel = in_channel.text();

            in_title.set_text("");
            in_channel.set_text("");

            let title_opt = if title.is_empty() { None } else { Some(title) };
            let channel_opt = if channel.is_empty() {
                None
            } else {
                Some(channel)
            };

            let title_regex = title_opt.map(|s| Regex::new(&s));
            let channel_regex = channel_opt.map(|s| Regex::new(&s));

            if let Some(Err(_)) = title_regex {
                // TODO: Error Handling
                return;
            }
            if let Some(Err(_)) = channel_regex {
                // TODO: Error Handling
                return;
            }

            filters
                .borrow()
                .as_ref()
                .expect("Filter List should be set up")
                .lock()
                .expect("Filter List should be lockable")
                .add(AnyVideoFilter::new(
                    None,
                    title_regex.map(|r| r.unwrap()),
                    channel_regex.map(|r| r.unwrap()),
                ));
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterPage {
        const NAME: &'static str = "TFFilterPage";
        type Type = super::FilterPage;
        type ParentType = StackPage;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FilterPage {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_toggle_add_filter(&self.obj());
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(Vec::new);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &glib::ParamSpec) {
            unimplemented!()
        }

        fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
            unimplemented!()
        }
    }

    impl WidgetImpl for FilterPage {}
    impl BinImpl for FilterPage {}
    impl StackPageImpl for FilterPage {}
}
