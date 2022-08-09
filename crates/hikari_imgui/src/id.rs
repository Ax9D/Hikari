use std::hash::Hash;

use imgui::{*};

pub trait IdCustomExt {
    fn new<T: Hash>(data: &T, ui: &Ui) -> Id;
}

impl IdCustomExt for Id {
    fn new<T: Hash>(data: &T, ui: &Ui) -> Id {
        Id::Int( fxhash::hash(data) as i32, ui)
    }
}