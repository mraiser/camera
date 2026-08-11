use ndata::dataobject::DataObject;
use ndata::data::Data;
use flowlang::datastore::DataStore;
use crate::camera::camera::settings::settings;
use flowlang::flowlang::system::time::time;
use std::path::Path;
use std::fs;
use chrono::Utc;
use chrono::TimeZone;
use std::path::PathBuf;

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        status()
    }));
    match ax {
        Ok(ax) => {
            let mut result_obj = DataObject::new();
    result_obj.put_object("a", ax);
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

pub fn status() -> DataObject {
let system = DataStore::globals().get_object("system");
let meta = system.get_object("apps").get_object("camera");
let settings = settings(Data::DNull);
let storage = settings.get_string("storage");

let recording = meta.has("last_keyframe") && time() - meta.get_int("last_keyframe") < 4000;

let mut o = settings.deep_copy();
o.put_object("app", meta.get_object("app"));
o.put_boolean("recording", recording);

let keyframes = Path::new(&storage).join("keyframes");
if keyframes.exists(){
  if let Some(v) = extreme(&keyframes, false) {
    let t = Utc.with_ymd_and_hms(v[0] as i32, v[1], v[2], v[3], v[4], v[5]*2).unwrap().timestamp_millis();
    o.put_int("first", t);
  }
  if let Some(v) = extreme(&keyframes, true) {
    let t = Utc.with_ymd_and_hms(v[0] as i32, v[1], v[2], v[3], v[4], v[5]*2).unwrap().timestamp_millis();
    o.put_int("last", t);
  }
}

o
}

// Walk keyframes/<year>/<month>/<day>/<hour>/<minute>/<slot>.jpg taking the
// numeric min (max=false) or max (max=true) entry at each level. Option
// instead of the old 0/99 sentinels: that scan treated 0 as not-found, so
// a most-recent recording at hour 0 / minute 0 / slot 0 (just after
// midnight) silently dropped `last`. The slot doubles into real seconds
// (the old code reported the raw slot index as the second).
fn extreme(root:&Path, max:bool) -> Option<[u32;6]> {
  let mut vals = [0u32;6];
  let mut dir = root.to_path_buf();
  let mut i = 0;
  while i < 6 {
    let mut best:Option<u32> = None;
    for f in fs::read_dir(&dir).ok()? {
      if f.is_err() { continue; }
      let p = f.unwrap().path();
      let name = match p.file_name() { Some(n) => n, None => continue };
      let name = match name.to_str() { Some(n) => n, None => continue };
      let name = if i == 5 { match name.strip_suffix(".jpg") { Some(s) => s, None => continue } } else { name };
      if let Ok(n) = name.parse::<u32>() {
        best = Some(match best {
          None => n,
          Some(b) => if (max && n > b) || (!max && n < b) { n } else { b }
        });
      }
    }
    match best {
      Some(b) => { vals[i] = b; if i < 5 { dir = dir.join(b.to_string()); } }
      None => { return None; }
    }
    i += 1;
  }
  Some(vals)
}
