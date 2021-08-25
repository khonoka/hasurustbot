use std::path::Path;
use std::{collections::BTreeSet, io::BufReader};

use std::fs::{create_dir_all, File};
use std::io::prelude::*;

pub struct FileData {
    _file_name: String,
    pub content: Vec<String>,
    _if_unique: bool,
}

impl FileData {
    pub fn new<S>(file_name: S, if_unique: bool) -> Result<Self, std::io::Error>
    where
        S: Into<String>,
    {
        let file_name = file_name.into();
        let mut data = Self {
            _file_name: file_name.clone(),
            content: Vec::new(),
            _if_unique: if_unique,
        };
        if let Some(p) = Path::new(&file_name).parent() {
            create_dir_all(p).unwrap_or_default();
        }
        let file = match File::open(&file_name) {
            Ok(f) => f,
            Err(e) => {
                if let std::io::ErrorKind::NotFound = e.kind() {
                    File::create(&file_name)?
                } else {
                    return Err(e);
                }
            }
        };
        let buf = BufReader::new(file);
        data.content = buf
            .lines()
            .map(|l| l.expect("Could not parse line"))
            .collect();

        if if_unique {
            let mut set = BTreeSet::new();
            for s in data.content.iter() {
                set.insert(s);
            }
            data.content = set.iter().map(|s| String::from(*s)).collect();
            let mut file = File::create(&file_name)?;
            for s in data.content.iter() {
                file.write_all(format!("{}\n", s).as_bytes())?;
            }
            file.sync_all()?;
        }

        Ok(data)
    }
    pub fn save<S>(&mut self, content: S) -> Result<(), std::io::Error>
    where
        S: Into<String>,
    {
        self.content.push(content.into());
        let mut file = File::create(&self._file_name)?;
        for s in self.content.iter() {
            file.write_all(format!("{}\n", s).as_bytes())?;
        }
        file.sync_all()?;
        Ok(())
    }
    /*
        pub fn del(&mut self, index: usize) -> Result<(), std::io::Error> {
            self.content.swap_remove(index);
            let mut file = File::create(&self._file_name)?;
            for s in self.content.iter() {
                file.write_all(format!("{}\n", s).as_bytes())?;
            }
            file.sync_all()?;
            Ok(())
        }
    */
}
