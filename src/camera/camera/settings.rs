use ndata::dataobject::*;
use ndata::data::Data;
use flowlang::datastore::DataStore;
use flowlang::flowlang::file::write_properties::write_properties;
use crate::camera::camera::start_recording::start_recording;
use crate::camera::camera::stop_recording::stop_recording;
use std::thread;
use core::time::Duration;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_property("settings");
let ax = settings(a0);
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn settings(settings:Data) -> DataObject {
let mut x = DataStore::globals().get_object("system").get_object("apps").get_object("camera").get_object("runtime");

if x.has("dvr") && Data::as_string(x.get_property("dvr")) == "true".to_string(){ x.put_boolean("dvr", true); }
else { x.put_boolean("dvr", false); }

if x.has("motion") && Data::as_string(x.get_property("motion")) == "true".to_string(){ x.put_boolean("motion", true); }
else { x.put_boolean("motion", false); }

if x.has("motion_sensitivity") { x.put_int("motion_sensitivity", Data::as_string(x.get_property("motion_sensitivity")).parse::<i64>().unwrap()); }
else { x.put_int("motion_sensitivity", 8); }

if x.has("motion_noise_cancel") { x.put_int("motion_noise_cancel", Data::as_string(x.get_property("motion_noise_cancel")).parse::<i64>().unwrap()); }
else { x.put_int("motion_noise_cancel", 4); }

if x.has("framerate") { x.put_int("framerate", Data::as_string(x.get_property("framerate")).parse::<i64>().unwrap()); }
else { x.put_int("framerate", 30); }

if x.has("rotation") { x.put_int("rotation", Data::as_string(x.get_property("rotation")).parse::<i64>().unwrap()); }
else { x.put_int("rotation", 0); }

if !x.has("storage") { x.put_string("storage", "runtime/camera/storage"); }

let s = Data::as_string(settings);
if s.starts_with("{") {
  let s = DataObject::from_string(&s);
  for (k,v) in s.objects() { x.set_property(&k, v); }
  let propfile = DataStore::new().root.parent().unwrap().join("runtime").join("camera").join("botd.properties");
  write_properties(propfile.into_os_string().into_string().unwrap(), x.clone());
  
  if s.has("dvr") {
    if x.has("dvr") && x.get_boolean("dvr"){
      if s.get_boolean("dvr") { 
        stop_recording();
        thread::spawn(|| {
          thread::sleep(Duration::from_millis(2000));
          start_recording(); 
        });
      }
      else { stop_recording(); }
    }
    else {
      if s.get_boolean("dvr") { 
        thread::spawn(|| {
          thread::sleep(Duration::from_millis(2000));
          start_recording(); 
        });
      }
      else { stop_recording(); }
    }
  }
}

x
}

