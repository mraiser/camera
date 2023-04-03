use ndata::dataobject::*;
use flowlang::datastore::DataStore;

pub fn execute(_o: DataObject) -> DataObject {
let ax = stop_recording();
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn stop_recording() -> String {
let mut meta = DataStore::globals().get_object("system").get_object("apps").get_object("camera");
if meta.has("pid") {
  let pid = meta.get_string("pid");
  meta.put_boolean(&pid, false);
}
"OK".to_string()
}

