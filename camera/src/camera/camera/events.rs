use ndata::dataobject::DataObject;
use crate::camera::camera::settings::settings;
use std::path::Path;
use ndata::data::Data;
use ndata::dataarray::DataArray;
use std::path::PathBuf;
use std::fs;

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        events()
    }));
    match ax {
        Ok(ax) => {
            let mut result_obj = DataObject::new();
    result_obj.put_array("a", ax);
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

pub fn events() -> DataArray {
let settings = settings(Data::DNull);
let root = settings.get_string("storage");
let root = Path::new(&root).join("events");

let a = DataArray::new();
if root.exists() { find(root, a.clone()); }

// server-side sort by event time, ascending (was left to the client)
let mut v:Vec<(i64,DataObject)> = Vec::new();
for o in a.objects() {
  let o = o.object();
  let t = if o.has("time") { o.get_int("time") } else { 0 };
  v.push((t, o));
}
v.sort_by_key(|x| x.0);
let mut out = DataArray::new();
for (_t, o) in v { out.push_object(o); }
out
}

fn find(dir:PathBuf, mut list:DataArray) {
  if let Ok(rd) = fs::read_dir(dir) {
    for f in rd {
      if f.is_err() { continue; }
      let f = f.unwrap();
      let meta = match fs::metadata(f.path()) { Ok(m) => m, Err(_) => continue };
      if meta.is_dir() {
        find(f.path(), list.clone());
      }
      else {
        let name = match f.file_name().into_string() { Ok(n) => n, Err(_) => continue };
        if name.ends_with(".json") {
          if let Ok(s) = fs::read_to_string(f.path()) {
            // tolerate a torn write (crash mid-motion_end): skip a file
            // that does not parse instead of erroring the whole listing
            let o = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| DataObject::from_string(&s)));
            if let Ok(o) = o { list.push_object(o); }
          }
        }
      }
    }
  }
}
