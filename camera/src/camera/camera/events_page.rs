use ndata::dataobject::DataObject;
use crate::camera::camera::settings::settings;
use std::path::Path;
use ndata::data::Data;
use ndata::dataarray::DataArray;
use std::path::PathBuf;
use std::fs;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["since", "limit"] {
        if !o.has(p) {
            let mut e = DataObject::new();
            e.put_string("status", "err");
            e.put_string("msg", &format!("missing required parameter: {}", p));
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", e);
            return result_obj;
        }
    }
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let arg_0: i64 = o.get_int("since");
        let arg_1: i64 = o.get_int("limit");
        events_page(arg_0, arg_1)
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

pub fn events_page(since: i64, limit: i64) -> DataArray {
let settings = settings(Data::DNull);
let root = settings.get_string("storage");
let root = Path::new(&root).join("events");

let a = DataArray::new();
if root.exists() { find(root, a.clone()); }

// newest `limit` events at or after `since` (epoch ms; 0 = no floor),
// returned in ascending time order for direct display
let mut v:Vec<(i64,DataObject)> = Vec::new();
for o in a.objects() {
  let o = o.object();
  let t = if o.has("time") { o.get_int("time") } else { 0 };
  if t >= since { v.push((t, o)); }
}
v.sort_by_key(|x| x.0);
let n = v.len();
let skip = if limit > 0 && n > limit as usize { n - limit as usize } else { 0 };
let mut out = DataArray::new();
for (_t, o) in v.into_iter().skip(skip) { out.push_object(o); }
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
            let o = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| DataObject::from_string(&s)));
            if let Ok(o) = o { list.push_object(o); }
          }
        }
      }
    }
  }
}
