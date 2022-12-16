use std::{io::Read, io::Write, path::{Path, PathBuf}};

pub struct Mode {
    pub create: bool,
    pub create_new: bool,
    pub read: bool,
    pub write: bool,
    pub append: bool,
}
impl Mode {
    pub fn read_only() -> Self {
        Mode {
            create: false,
            create_new: false,
            read: true,
            write: false,
            append: false,
        }
    }
    pub fn create_and_write() -> Self {
        Mode {
            create: true,
            create_new: false,
            read: false,
            write: true,
            append: false,
        }
    }
}
pub trait IO: Send + Sync + 'static {
    fn read_file(
        &self,
        path: &Path,
        mode: &Mode,
    ) -> Result<Box<dyn Read + Send + Sync + 'static>, std::io::Error>;
    fn write_file(
        &self,
        path: &Path,
        mode: &Mode,
    ) -> Result<Box<dyn Write + Send + Sync + 'static>, std::io::Error>;

    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error>;
    fn rename_file(&self, old: &Path, new: &Path) -> Result<(), std::io::Error>;

    fn create_temp_file(&self, path: &Path, mode: &Mode) -> Result<(PathBuf, Box<dyn Write + Send + Sync + 'static>), std::io::Error> {
        loop {
            let random = rand::random::<u32>().to_string();
            let mut temp_path = path.to_owned();
            temp_path.set_extension(Path::new(&random));

            let result = self.write_file(&temp_path, &Mode {
                create: true,
                create_new: true,
                read: mode.read, 
                write: mode.write,
                append: mode.append,
            });
            
            match result {
                Ok(writer) => {
                    return Ok((temp_path, writer));
                },
                Err(err) => {
                    match err.kind() {
                        std::io::ErrorKind::AlreadyExists => continue,
                        _ => return Err(err),
                    }
                }
            }
        } 
    }
}


pub struct PhysicalIO;

impl IO for PhysicalIO {
    fn read_file(
        &self,
        path: &Path,
        mode: &Mode,
    ) -> Result<Box<dyn Read + Send + Sync + 'static>, std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .create(mode.create)
            .create_new(mode.create_new)
            .read(mode.read)
            .write(mode.write)
            .append(mode.append)
            .open(path)?;

        Ok(Box::new(file))
    }

    fn write_file(
        &self,
        path: &Path,
        mode: &Mode,
    ) -> Result<Box<dyn Write + Send + Sync + 'static>, std::io::Error> {
        let file = std::fs::OpenOptions::new()
            .create(mode.create)
            .create_new(mode.create_new)
            .read(mode.read)
            .write(mode.write)
            .append(mode.append)
            .open(path)?;

        Ok(Box::new(file))
    }

    fn remove_file(&self, path: &Path) -> Result<(), std::io::Error> {
        std::fs::remove_file(path)
    }

    fn rename_file(&self, from: &Path, to: &Path) -> Result<(), std::io::Error> {
        std::fs::rename(from, to)
    }
}
