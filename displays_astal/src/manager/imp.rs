use glib::subclass::prelude::*;

#[derive(Default)]
pub struct Manager;

#[glib::object_subclass]
impl ObjectSubclass for Manager {
    const NAME: &'static str = "AstalDisplaysManager";
    type Type = super::Manager;
}

impl ObjectImpl for Manager {}
