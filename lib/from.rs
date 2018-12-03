extern crate serde;
extern crate serde_json;

use error::CSDError;
use exec::*;

use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;

use self::serde::de::DeserializeOwned;

// Generic trait to read file to serializable struct
pub trait FromFile<T> {
    fn from_file(path: &str) -> Result<T, CSDError>;
}

impl<T: DeserializeOwned + Debug> FromFile<T> for T {
    fn from_file(path: &str) -> Result<T, CSDError> {
        let mut file = File::open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(serde_json::from_str(&buffer)?)
    }
}

// Generic trait to create structs from ceph JSON output. Since most if not all
// of of ceph's commands can be formatted to JSON. For example:
// let pgmap = PGMap::from_ceph("pg dump").unwrap()
pub trait FromCeph<T> {
    fn from_ceph(cmd: &str) -> Result<T, CSDError>;
}

impl<T: DeserializeOwned + Debug> FromCeph<T> for T {
    fn from_ceph(cmd: &str) -> Result<T, CSDError> {
        Ok(serde_json::from_str(&call_ceph(cmd)?)?)
    }
}
