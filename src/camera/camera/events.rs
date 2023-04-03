use ndata::dataobject::*;
use crate::camera::camera::settings::settings;
use std::path::Path;
use ndata::data::Data;
use ndata::dataarray::DataArray;
use std::path::PathBuf;
use std::fs;

pub fn execute(_o: DataObject) -> DataObject {
let ax = events();
let mut o = DataObject::new();
o.put_array("a", ax);
o
}

pub fn events() -> DataArray {
  let settings = settings(Data::DNull);
  let root = settings.get_string("storage");
  let root = Path::new(&root).join("events");

  let a = DataArray::new();
  find(root, a.clone());
  // FIXME - sort by key "time"
  a
}

fn find(dir:PathBuf, mut list:DataArray) {
  for f in fs::read_dir(dir).unwrap(){
    let f = f.unwrap();
    if fs::metadata(f.path()).unwrap().is_dir(){
      find(f.path(), list.clone());
    }
    else {
      let name = f.file_name().into_string().unwrap();
      if name.ends_with(".json") {
        let s = fs::read_to_string(f.path()).unwrap();
        let o = DataObject::from_string(&s);
        list.push_object(o);
      }
    }
  }

}

