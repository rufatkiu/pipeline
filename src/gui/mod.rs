use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use tf_observer::Observer;

mod feed;
mod filter;
mod import_window;
mod predefined_piped_api;
mod predefined_player;
mod preferences_window;
mod stack_page;
mod subscription;
mod utility;
mod video_information_window;
mod watch_later;
pub mod window;

pub(crate) type BoxedObserver<T> = RefCell<Option<Arc<Mutex<Box<dyn Observer<T> + Send>>>>>;
