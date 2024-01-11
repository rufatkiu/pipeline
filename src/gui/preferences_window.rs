use gdk::glib::Object;

gtk::glib::wrapper! {
    pub struct PreferencesWindow(ObjectSubclass<imp::PreferencesWindow>)
        @extends libadwaita::PreferencesWindow, libadwaita::Window, gtk::Window, gtk::Widget,
        @implements gtk::gio::ActionGroup, gtk::gio::ActionMap, gtk::Accessible, gtk::Buildable,
            gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl PreferencesWindow {
    pub fn new() -> Self {
        Object::builder().build()
    }
}

pub mod imp {
    use gdk::gio::Settings;
    use gdk::gio::SettingsBindFlags;
    use gdk_pixbuf::gio::ListStore;
    use gdk_pixbuf::glib::ParamSpec;
    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::CompositeTemplate;
    use gtk::Switch;
    use libadwaita::prelude::*;
    use libadwaita::subclass::prelude::AdwWindowImpl;
    use libadwaita::subclass::prelude::PreferencesWindowImpl;
    use libadwaita::traits::PreferencesGroupExt;
    use libadwaita::EntryRow;

    use crate::gui::predefined_piped_api::PredefinedPipedApi;
    use crate::gui::predefined_player::PredefinedPlayer;

    #[derive(CompositeTemplate)]
    #[template(resource = "/ui/preferences_window.ui")]
    pub struct PreferencesWindow {
        #[template_child]
        entry_player: TemplateChild<EntryRow>,
        #[template_child]
        entry_downloader: TemplateChild<EntryRow>,

        #[template_child]
        combo_predefined_player: TemplateChild<libadwaita::ComboRow>,
        #[template_child]
        combo_predefined_piped_api: TemplateChild<libadwaita::ComboRow>,

        #[template_child]
        entry_piped_api: TemplateChild<EntryRow>,

        #[template_child]
        group_programs: TemplateChild<libadwaita::PreferencesGroup>,

        #[template_child]
        switch_only_videos_yesterday: TemplateChild<Switch>,

        settings: Settings,
    }

    #[gtk::template_callbacks]
    impl PreferencesWindow {
        fn predefined_players_vec(&self) -> Vec<PredefinedPlayer> {
            vec![
                PredefinedPlayer::new("MPV (Flatpak)", "flatpak run io.mpv.Mpv"),
                PredefinedPlayer::new("MPV", "mpv"),
                PredefinedPlayer::new(
                    "Clapper (Flatpak)",
                    "flatpak run com.github.rafostar.Clapper",
                ),
                PredefinedPlayer::new("Clapper", "clapper"),
                PredefinedPlayer::new(
                    "Celluloid (Flatpak)",
                    "flatpak run io.github.celluloid_player.Celluloid",
                ),
                PredefinedPlayer::new("Celluloid", "celluloid"),
                PredefinedPlayer::new("Livi", "livi --yt-dlp"),
                PredefinedPlayer::new("Custom", ""),
            ]
        }

        #[template_callback]
        fn predefined_players(&self) -> ListStore {
            let store = ListStore::new::<PredefinedPlayer>();
            store.extend_from_slice(&self.predefined_players_vec());
            store
        }

        fn predefined_piped_apis_vec(&self) -> Vec<PredefinedPipedApi> {
            vec![
                PredefinedPipedApi::new("kavin.rocks (official)", "https://pipedapi.kavin.rocks"),
                PredefinedPipedApi::new(
                    "kavin.rocks (libre, official)",
                    "https://pipedapi-libre.kavin.rocks",
                ),
                PredefinedPipedApi::new("tokhmi.xyz", "https://pipedapi.tokhmi.xyz"),
                PredefinedPipedApi::new("lunar.icu", "https://piped-api.lunar.icu"),
                PredefinedPipedApi::new("Custom", ""),
            ]
        }

        #[template_callback]
        fn predefined_piped_apis(&self) -> ListStore {
            let store = ListStore::new::<PredefinedPipedApi>();
            store.extend_from_slice(&self.predefined_piped_apis_vec());
            store
        }

        #[template_callback]
        fn handle_selection_player(&self, _: ParamSpec, r: libadwaita::ComboRow) {
            if let Ok(player) = r.selected_item().and_dynamic_cast::<PredefinedPlayer>() {
                self.entry_player.set_visible(player.command().is_empty());
                if !player.command().is_empty() {
                    self.entry_player.set_text(&player.command());
                }
            }
        }

        #[template_callback]
        fn handle_selection_piped_api(&self, _: ParamSpec, r: libadwaita::ComboRow) {
            if let Ok(api) = r.selected_item().and_dynamic_cast::<PredefinedPipedApi>() {
                self.entry_piped_api.set_visible(api.url().is_empty());
                if !api.url().is_empty() {
                    self.entry_piped_api.set_text(&api.url());
                }
            }
        }

        fn init_flatpak(&self) {
            self.group_programs.set_description(Some(&gettextrs::gettext("Note that on Flatpak, there are some more steps required when using a player external to the Flatpak. For more information, please consult the wiki.")));
        }

        fn init_predefined_player(&self) {
            let val_env = std::env::var_os("PLAYER");
            let val_settings = self.settings.string("player");
            if val_env.is_some() && &val_env.unwrap() != val_settings.as_str() {
                self.combo_predefined_player.set_sensitive(false);
            }
            if let Some(idx) = self
                .predefined_players_vec()
                .iter()
                .position(|o| o.command() == val_settings)
            {
                self.combo_predefined_player
                    .set_selected(idx.try_into().unwrap_or_default());
                self.entry_player.set_visible(false);
            } else {
                // Select "Custom".
                self.combo_predefined_player
                    .set_selected(self.predefined_players_vec().len().try_into().unwrap_or(1) - 1);
            }
        }

        fn init_predefined_piped_api(&self) {
            let val_env = std::env::var_os("PIPED_API_URL");
            let val_settings = self.settings.string("piped-url");
            if val_env.is_some() && &val_env.unwrap() != val_settings.as_str() {
                self.combo_predefined_player.set_sensitive(false);
            }
            if let Some(idx) = self
                .predefined_piped_apis_vec()
                .iter()
                .position(|o| o.url() == val_settings)
            {
                self.combo_predefined_piped_api
                    .set_selected(idx.try_into().unwrap_or_default());
                self.entry_piped_api.set_visible(false);
            } else {
                // Select "Custom".
                self.combo_predefined_piped_api.set_selected(
                    self.predefined_piped_apis_vec()
                        .len()
                        .try_into()
                        .unwrap_or(1)
                        - 1,
                );
            }
        }

        fn init_string_setting(&self, env: &'static str, settings: &'static str, entry: EntryRow) {
            let val_env = std::env::var_os(env);
            let val_settings = self.settings.string(settings);
            entry.set_text(&val_settings);
            if val_env.is_some() && &val_env.unwrap() != val_settings.as_str() {
                entry.set_editable(false);
            }
            self.settings
                .bind(settings, &entry, "text")
                .flags(SettingsBindFlags::DEFAULT)
                .build();
            entry.connect_changed(move |entry| std::env::set_var(env, entry.text()));
        }

        fn init_settings(&self) {
            self.init_string_setting("PLAYER", "player", self.entry_player.get());
            self.init_string_setting("DOWNLOADER", "downloader", self.entry_downloader.get());
            self.init_string_setting("PIPED_API_URL", "piped-url", self.entry_piped_api.get());

            self.settings
                .bind(
                    "only-videos-yesterday",
                    &self.switch_only_videos_yesterday.get(),
                    "active",
                )
                .flags(SettingsBindFlags::DEFAULT)
                .build();

            self.init_predefined_player();
            self.init_predefined_piped_api();
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PreferencesWindow {
        const NAME: &'static str = "TFPreferencesWindow";
        type Type = super::PreferencesWindow;
        type ParentType = libadwaita::PreferencesWindow;

        fn new() -> Self {
            Self {
                settings: Settings::new(crate::config::APP_ID),
                group_programs: TemplateChild::default(),
                entry_player: TemplateChild::default(),
                entry_downloader: TemplateChild::default(),
                entry_piped_api: TemplateChild::default(),
                combo_predefined_player: Default::default(),
                combo_predefined_piped_api: Default::default(),
                switch_only_videos_yesterday: Default::default(),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
            Self::bind_template_callbacks(klass);
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for PreferencesWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.init_settings();
            if crate::config::FLATPAK {
                self.init_flatpak();
            }
        }
    }
    impl WidgetImpl for PreferencesWindow {}
    impl WindowImpl for PreferencesWindow {}
    impl PreferencesWindowImpl for PreferencesWindow {}
    impl AdwWindowImpl for PreferencesWindow {}
}
