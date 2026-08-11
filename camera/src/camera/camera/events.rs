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
