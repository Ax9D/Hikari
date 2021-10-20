use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{atomic::AtomicU32, Arc},
};

use parking_lot::Mutex;

static GENERATION_COUNT: AtomicU32 = AtomicU32::new(0);

// struct Index {
//     index: u32,
//     generation: u32,
// }
#[derive(Clone)]
pub struct ResourceInfo {
    name: String,
    path: Option<PathBuf>,
}
impl ResourceInfo {
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        let name = name.as_ref().to_owned();
        Self { name, path: None }
    }
    pub fn new_with_path<S: AsRef<str>, P: AsRef<Path>>(name: S, path: P) -> Self {
        let name = name.as_ref().to_owned();
        let path = path.as_ref().to_owned();

        Self {
            name,
            path: Some(path),
        }
    }
}
struct AssociatedData {
    info: ResourceInfo,
    index: u32,
    free_list: Arc<Mutex<Vec<usize>>>,
}
impl Drop for AssociatedData {
    fn drop(&mut self) {
        self.free_list.lock().push(self.index as usize);
    }
}

unsafe impl<T> Sync for Handle<T> {}
unsafe impl<T> Send for Handle<T> {}

#[derive(Clone)]
pub struct Handle<T> {
    index: u32,
    generation: u32,
    data: Arc<UnsafeCell<AssociatedData>>,

    _phantom: PhantomData<T>,
}
impl<T> Handle<T> {
    pub(super) fn new(
        info: ResourceInfo,
        index: usize,
        generation: u32,
        free_list: Arc<Mutex<Vec<usize>>>,
    ) -> Self {
        let index = index as u32;
        Self {
            index,
            generation,
            data: Arc::new(UnsafeCell::new(AssociatedData {
                info,
                index,
                free_list,
            })),
            _phantom: PhantomData::default(),
        }
    }
    #[inline]
    pub(super) fn index(&self) -> usize {
        self.index as usize
    }
    #[inline]
    pub(super) fn generation(&self) -> u32 {
        self.generation
    }
}

impl<T> Handle<T> {
    #[inline]
    pub fn read<'a>(&self, pool: &'a ResourcePool<T>) -> &'a T {
        pool.get(self).unwrap()
    }
    #[inline]
    pub fn write<'a>(&self, pool: &'a mut ResourcePool<T>) -> &'a mut T {
        pool.get_mut(self).unwrap()
    }

    #[inline]
    fn data(&self) -> &AssociatedData {
        unsafe { &*self.data.get() }
    }
    #[inline]
    unsafe fn data_mut(&mut self) -> &mut AssociatedData {
        unsafe { &mut *self.data.get() }
    }
    pub fn name(&self) -> &str {
        &self.data().info.name
    }

    pub fn path(&self) -> Option<&Path> {
        self.data().info.path.as_deref()
    }
}
pub struct ResourcePool<T> {
    generation: u32,
    data: Vec<Option<T>>,
    free_list: Arc<Mutex<Vec<usize>>>,
}

// trait Threadsafe: Send + Sync {}

impl<T> ResourcePool<T> {
    pub fn new() -> Self {
        const DEFAULT_INITIAL_CAPACITY: usize = 4;

        Self::with_capacity(DEFAULT_INITIAL_CAPACITY)
    }
    pub fn with_capacity(n: usize) -> Self {
        let generation = GENERATION_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let data = Vec::with_capacity(n);

        let free_list = Vec::new();
        let free_list = Arc::new(Mutex::new(free_list));

        Self {
            generation,
            data,
            free_list,
        }
    }
    pub fn add(&mut self, resource_info: ResourceInfo, data: T) -> Handle<T> {
        if let Some(new_index) = self.free_list.lock().pop() {
            self.data[new_index] = Some(data);

            Handle::new(
                resource_info,
                new_index,
                self.generation,
                self.free_list.clone(),
            )
        } else {
            let new_index = self.data.len();
            self.data.push(Some(data));

            Handle::new(
                resource_info,
                new_index,
                self.generation,
                self.free_list.clone(),
            )
        }
    }
    // pub unsafe fn get_unchecked(&self, handle: &Handle<T>) -> &T {
    //     assert!(handle.generation() == self.generation);
    //     self.data.get_unchecked(handle.index())
    // }

    #[inline]
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        if handle.generation() == self.generation {
            self.data.get(handle.index()).unwrap().as_ref()
        } else {
            None
        }
    }
    // pub unsafe fn get_unchecked_mut(&mut self, handle: &Handle<T>) -> &mut T{
    //     //assert!(handle.parent() == self.generation);
    //     self.data.get_unchecked_mut(handle.index())
    // }

    #[inline]
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        if handle.generation() == self.generation {
            self.data.get_mut(handle.index()).unwrap().as_mut()
        } else {
            None
        }

        // match self.data.get_mut(handle.index()) {
        //     Some(data) if handle.generation() == self.generation => Some(data),
        //     _=> None
        // }
    }

    ///Repalces the underlying data keeping the same handle
    pub fn replace_data(
        &mut self,
        handle: &mut Handle<T>,
        new_resource_info: ResourceInfo,
        new_data: T,
    ) {
        if let Some(data) = self.get_mut(handle) {
            *data = new_data;

            //Have exclusive access to pool and handle so it's safe
            unsafe { handle.data_mut() }.info = new_resource_info;
        }
    }

    pub unsafe fn raw_data(&self) -> &Vec<Option<T>> {
        &self.data
    }

    ///Releases resources which do not have a handle to them anymore
    pub fn garbage_collect(&mut self) {
        for &free_index in self.free_list.lock().iter() {
            self.data[free_index] = None; // Run drop
        }
    }
}
use std::ops::Index;

impl<T> Index<&Handle<T>> for ResourcePool<T> {
    type Output = T;

    fn index(&self, handle: &Handle<T>) -> &Self::Output {
        self.get(handle).unwrap()
    }
}
use std::ops::IndexMut;
impl<T> IndexMut<&Handle<T>> for ResourcePool<T> {
    fn index_mut(&mut self, handle: &Handle<T>) -> &mut Self::Output {
        self.get_mut(handle).unwrap()
    }
}
#[cfg(test)]
mod tests {
    use std::{ops::DerefMut, time::Instant};
    extern crate test;
    use test::Bencher;

    use crate::Asset;

    use super::*;
    use rand::Rng;
    extern crate rand;
    const N: usize = 100;
    const PATH: &'static str = r"E:\FlightHelmet\FlightHelmet.gltf";
    #[bench]
    fn shared_mut_materials(b: &mut Bencher) -> Result<(), Box<dyn std::error::Error>> {
        let mut scene = crate::Scene::load(PATH)?;
        let material_ix = scene.models[0].meshes[0].material.unwrap();

        let mut mats = Vec::new();
        for _ in 0..N {
            mats.push(Asset::new(
                "",
                "",
                scene.materials[rand::thread_rng().gen_range(0..scene.materials.len())].clone(),
            ));
        }
        for mat in scene.materials {
            mats.push(Asset::new("", "", mat));
        }

        use std::ops::Deref;
        b.iter(|| {
            for mat in &mut mats {
                mat.albedo = glam::vec4(
                    rand::random(),
                    rand::random(),
                    rand::random(),
                    rand::random(),
                );
            }
        });

        println!("{}", mats.first().unwrap().name());
        Ok(())
    }

    #[bench]
    fn pool_materials(b: &mut Bencher) -> Result<(), Box<dyn std::error::Error>> {
        let mut scene = crate::Scene::load(PATH)?;

        let mut pool = ResourcePool::new();

        let mut mats = Vec::new();
        for _ in 0..N {
            mats.push(pool.add(
                ResourceInfo::new("Test"),
                scene.materials[rand::thread_rng().gen_range(0..scene.materials.len())].clone(),
            ));
        }
        for mat in scene.materials {
            mats.push(pool.add(ResourceInfo::new(mat.name()), mat))
        }

        b.iter(|| {
            for mat in &mut mats {
                let mat = &mut pool[mat];
                mat.albedo = glam::vec4(
                    rand::random(),
                    rand::random(),
                    rand::random(),
                    rand::random(),
                );
            }
        });

        println!("{}", mats.first().unwrap().read(&pool).name());
        Ok(())
    }
    #[test]
    fn shared_mut() {
        let mut mats = Vec::new();
        for _ in 0..N {
            mats.push(Asset::new(
                "Test",
                "",
                crate::Material {
                    name: "()".into(),
                    albedo: glam::Vec4::ZERO,
                    albedo_map: None,
                    roughness: rand::random(),
                    roughness_map: None,
                    metallic: rand::random(),
                    metallic_map: None,
                    normal_map: None,
                },
            ));
        }

        let now = Instant::now();

        for mat in &mut mats {
            //let mut mat = mat.lock();
            //let now = Instant::now();

            let mat = mat.deref_mut();

            //println!("Shared mut {:?}", now.elapsed());

            mat.albedo = glam::vec4(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            );
        }

        println!(
            "\n shared_mut {:?} {:?}\n",
            now.elapsed(),
            mats[rand::thread_rng().gen_range(0..mats.len())].albedo
        );
    }
    #[test]
    fn pool() {
        let mut pool = ResourcePool::new();
        let mut mats = Vec::new();

        for _ in 0..N {
            mats.push(pool.add(
                ResourceInfo::new("Test"),
                crate::Material {
                    name: "()".into(),
                    albedo: glam::Vec4::ZERO,
                    albedo_map: None,
                    roughness: rand::random(),
                    roughness_map: None,
                    metallic: rand::random(),
                    metallic_map: None,
                    normal_map: None,
                },
            ));
        }

        let now = Instant::now();

        for mat in &mats {
            //let now = Instant::now();

            let mat = &mut pool[mat];

            //println!("Pool {:?}", now.elapsed());

            mat.albedo = glam::vec4(
                rand::random(),
                rand::random(),
                rand::random(),
                rand::random(),
            );
        }
        println!(
            "\n pool {:?} {:?}\n",
            now.elapsed(),
            mats[rand::thread_rng().gen_range(0..mats.len())]
                .read(&pool)
                .albedo
        );
    }
    #[test]
    fn add_in_loop() {
        let mut pool = ResourcePool::new();
        for _ in 0..1000 {
            pool.add(ResourceInfo::new("test"), "Test data");
            pool.garbage_collect();
        }

        unsafe {
            println!("Length of pool {}", pool.raw_data().len());

            assert!(std::mem::size_of::<Handle<()>>() == 16);
            println!("size of Handle = {}", std::mem::size_of::<Handle<()>>());
        }
    }
}
