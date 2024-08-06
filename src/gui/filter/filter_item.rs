use std::sync::{Arc, Mutex};

use gdk::subclass::prelude::ObjectSubclassIsExt;
use gtk::glib::Object;
use tf_filter::FilterGroup;
use tf_join::AnyVideoFilter;

gtk::glib::wrapper! {
    pub struct FilterItem(ObjectSubclass<imp::FilterItem>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget;
}

impl FilterItem {
    pub fn new(filter_group: Arc<Mutex<FilterGroup<AnyVideoFilter>>>) -> Self {
        let s: Self = Object::builder::<Self>().build();
        s.imp().filter_group.replace(Some(filter_group));
        s
    }
}

pub mod imp {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::sync::Mutex;

    use gdk::glib::clone;
    use gdk::glib::ParamSpecObject;
    use gdk::glib::Value;
    use gdk_pixbuf::gio::SimpleAction;
    use gdk_pixbuf::gio::SimpleActionGroup;
    use glib::subclass::InitializingObject;
    use glib::ParamSpec;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use once_cell::sync::Lazy;
    use tf_filter::FilterGroup;
    use tf_join::AnyVideoFilter;

    use crate::gui::filter::filter_item_object::FilterObject;
    use crate::gui::utility::Utility;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/ui/filter_item.ui")]
    pub struct FilterItem {
        #[template_child]
        label_title: TemplateChild<gtk::Label>,
        #[template_child]
        label_channel: TemplateChild<gtk::Label>,
        #[template_child]
        popover_menu: TemplateChild<gtk::Popover>,

        filter: RefCell<Option<FilterObject>>,
        pub(super) filter_group: RefCell<Option<Arc<Mutex<FilterGroup<AnyVideoFilter>>>>>,
    }

    #[gtk::template_callbacks]
    impl FilterItem {
        #[template_callback]
        fn handle_clicked(&self) {
            self.popover_menu.popup();
        }

        fn setup_actions(&self, obj: &super::FilterItem) {
            let action_remove = SimpleAction::new("remove", None);
            action_remove.connect_activate(clone!(
                #[weak]
                obj,
                move |_, _| {
                    let filter = obj.imp().filter.borrow().as_ref().and_then(|s| s.filter());
                    let filter_group = obj.imp().filter_group.borrow_mut();
                    if let Some(filter) = filter {
                        filter_group
                            .as_ref()
                            .expect("FilterGroup to be set up")
                            .lock()
                            .expect("FilterGroup to be lockable")
                            .remove(&filter);
                    }
                }
            ));

            let actions = SimpleActionGroup::new();
            obj.insert_action_group("item", Some(&actions));
            actions.add_action(&action_remove);
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterItem {
        const NAME: &'static str = "TFFilterItem";
        type Type = super::FilterItem;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
            Utility::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FilterItem {
        fn constructed(&self) {
            self.parent_constructed();
            self.setup_actions(&self.obj());
        }

        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> =
                Lazy::new(|| vec![ParamSpecObject::builder::<FilterObject>("filter").build()]);
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "filter" => {
                    let value: Option<FilterObject> =
                        value.get().expect("Property filter of incorrect type");
                    self.filter.replace(value);
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "filter" => self.filter.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for FilterItem {}
    impl BoxImpl for FilterItem {}
}
