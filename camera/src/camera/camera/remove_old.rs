use ndata::dataobject::DataObject;
use crate::camera::camera::settings::settings;
use std::path::Path;
use ndata::data::Data;
use flowlang::flowlang::system::time::time;
use std::fs;
use std::time::UNIX_EPOCH;

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        remove_old()
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

pub fn remove_old() -> String {
let settings = settings(Data::DNull);
let storage = settings.get_string("storage");
let path = Path::new(&storage);
let days;
if settings.has("days") { days = settings.get_string("days").parse::<f64>().unwrap(); }
else { days = 2.0; }
let millis = (days * 24.0 * 60.0 * 60.0 * 1000.0) as i64;
let time = time() - millis;

cull(&path.join("archive"), time);
cull(&path.join("events"), time);
cull(&path.join("generated"), time);
cull(&path.join("keyframes"), time);

"OK".to_string()
}

fn cull(path:&Path, time:i64) {
  if path.exists() {
    for file in fs::read_dir(&path).unwrap() {
      let file = file.unwrap();
      let path = file.path();
      let metadata = fs::metadata(path.to_owned()).unwrap();
      if metadata.is_dir() {
        cull(&path, time);
      }
      else if metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis() < time as u128 {
        let _x = fs::remove_file(&path);
      }
    }
    if path.read_dir().unwrap().next().is_none() {
      let _x = fs::remove_dir_all(&path);
    }
  }
}
