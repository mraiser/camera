use ndata::dataobject::*;
use flowlang::datastore::DataStore;

pub fn execute(_o: DataObject) -> DataObject {
let ax = last_keyframe();
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn last_keyframe() -> String {
let system = DataStore::globals().get_object("system");
let meta = system.get_object("apps").get_object("camera");
if meta.has("last_jpg") {
//  if time() - 4000 < meta.get_i64("last_keyframe") {
    return meta.get_string("last_jpg");
//  }
}
return DataStore::new().root.join("camera").join("_ASSETS").join("no_signal.jpg").into_os_string().into_string().unwrap();
}

