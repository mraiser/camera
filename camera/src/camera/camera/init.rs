use ndata::dataobject::DataObject;
use crate::camera::camera::start_recording::start_recording;
use std::thread;
use crate::camera::camera::settings::settings;
use ndata::data::Data;
use crate::camera::camera::remove_old::remove_old;
use flowlang::datastore::DataStore;
use core::time::Duration;
use ndata::dataarray::*;
use flowlang::flowlang::data::write::write;

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        init()
    }));
    match ax {
        Ok(ax) => {
            let mut result_obj = DataObject::new();
    result_obj.put_string("a", &ax);
            result_obj
        }
        Err(err) => {
            let mut err_obj = DataObject::new();
            err_obj.put_string("status", "err");

            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };

            err_obj.put_string("msg", &msg);
            // Wrapped in the same `a` envelope a successful return uses.
            // Unwrapped, callers that unpack the envelope (newbound's
            // format_result, for one) report an opaque 500 — "Not an object:
            // DString(\"err\")" — instead of this message.
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", err_obj);
            result_obj
        }
    }
}

pub fn init() -> String {
let settings = settings(Data::DNull);
if settings.has("dvr") && settings.get_boolean("dvr"){
  thread::spawn(move || {
    start_recording();
  });
  thread::spawn(move || {
    let system = DataStore::globals().get_object("system");
    let beat = Duration::from_millis(60000);
    while system.get_boolean("running") {
      remove_old();
      thread::sleep(beat);
    }
  });
}

let store = DataStore::new();
let mut d;
if store.exists("runtime", "controls_available") { d = store.get_data("runtime", "controls_available").get_object("data"); }
else { d = DataObject::new(); }

let mut b = false;
if !d.has("camera:dvr") {
  let o = DataObject::from_string("{\"title\":\"Camera/DVR\",\"type\":\"camera:dvr\",\"big\":true,\"position\":\"inline\",\"groups\":[\"admin\"]}");
  d.put_object("camera:dvr", o);
  b = true;
}

if b { write("runtime".to_string(), "controls_available".to_string(), d.clone(), DataArray::new(), DataArray::new()); }

"OK".to_string()
}
