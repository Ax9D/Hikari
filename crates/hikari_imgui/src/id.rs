use std::hash::Hash;

use imgui::*;

pub trait IdCustomExt {
    fn new<T: Hash>(data: &T, ui: &Ui) -> Id;
}

impl IdCustomExt for Id {
    fn new<T: Hash>(data: &T, ui: &Ui) -> Id {
        ui.new_id_int(fxhash::hash(data) as i32)
    }
}
