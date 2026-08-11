use ndata::dataobject::DataObject;
use chrono::DateTime;
use chrono::Utc;
use chrono::TimeZone;
use chrono::Datelike;
use chrono::Timelike;
use std::path::Path;
use crate::camera::camera::settings::settings;
use ndata::data::Data;
use flowlang::datastore::DataStore;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["timestamp"] {
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
        let arg_0: String = o.get_string("timestamp");
        keyframe(arg_0)
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

pub fn keyframe(timestamp: String) -> String {
// timestamp arrives raw off the URL (the DVR picker can emit NaN when
// the archive is empty): parse leniently, soft-fail to no-signal.
let timestamp = match timestamp.trim().parse::<i64>() {
  Ok(t) => t,
  Err(_) => { return DataStore::new().root.join("camera").join("_ASSETS").join("no_signal.jpg").into_os_string().into_string().unwrap(); }
};
let settings = settings(Data::DNull);
let storage = settings.get_string("storage");

let now:DateTime<Utc> = Utc.timestamp_millis_opt(timestamp).unwrap();
let year = now.year();
let month = now.month();
let day = now.day();
let hour = now.hour();
let minute = now.minute();
let second = now.second();
let index = second / 2;

Path::new(&storage)
	.join("keyframes")
    .join(year.to_string())
    .join(month.to_string())
    .join(day.to_string())
    .join(hour.to_string())
    .join(minute.to_string())
    .join(index.to_string()+".jpg")
    .into_os_string()
    .into_string()
    .unwrap()
}
