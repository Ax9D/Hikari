use std::{path::{Path, PathBuf, Display}};

// #[derive(Clone, Debug)]
// pub struct AssetPath {
//     asset_dir: PathBuf,
//     absolute_asset_path: PathBuf
// }
// impl AssetPath {
//     pub fn new(asset_dir: &Path, path: &Path) -> Self {
//         let absolute_asset_path = if path.is_relative() {
//             asset_dir.join(path)
//         }
//         else {
//             path.to_owned()
//         };
//         Self { asset_dir: asset_dir.to_owned(), absolute_asset_path }
//     } 
//     pub fn absolute_path(&self) -> &Path {
//         &&self.absolute_asset_path
//     }
//     pub fn referenceable_path(&self) -> PathBuf {
//         self.absolute_asset_path.strip_prefix(&self.asset_dir).ok().map(|path| path.to_owned()).unwrap_or(self.absolute_asset_path.clone())
//     }
//     pub fn display(&self) -> Display {
//         self.absolute_asset_path.display()
//     }
// }


pub fn add_extension(path: &Path, extension: impl AsRef<std::path::Path>) -> PathBuf {
    let mut path = path.to_owned();
    match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_os_string();
            ext.push(".");
            ext.push(extension.as_ref());
            path.set_extension(ext)
        }
        None => path.set_extension(extension.as_ref()),
    };

    path
}

#[inline]
pub fn make_relative(original: &Path, relative_from: &Path) -> Option<PathBuf> {
    path_relative_from(original, relative_from)
}
#[test]
fn make_relative_test() {
    assert_eq!(Some(PathBuf::from("../HikariProjs/Sponza")), make_relative(Path::new("/home/atri/HikariProjs/Sponza"), Path::new("/home/atri/Hikari")));
}
// Copyright 2012-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//https://github.com/rust-lang/rust/blob/e1d0de82cc40b666b88d4a6d2c9dcbc81d7ed27f/src/librustc_back/rpath.rs#L116-L158
// This routine is adapted from the *old* Path's `path_relative_from`
// function, which works differently from the new `relative_from` function.
// In particular, this handles the case on unix where both paths are
// absolute but with only the root as the common directory.
fn path_relative_from(path: &Path, base: &Path) -> Option<PathBuf> {
    use std::path::Component;

    if path.is_absolute() != base.is_absolute() {
        if path.is_absolute() {
            Some(PathBuf::from(path))
        } else {
            None
        }
    } else {
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component> = vec![];
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => (),
                (Some(a), Some(b)) if b == Component::CurDir => comps.push(a),
                (Some(_), Some(b)) if b == Component::ParentDir => return None,
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().map(|c| c.as_os_str()).collect())
    }
}
