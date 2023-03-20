let system = DataStore::globals().get_object("system");
let meta = system.get_object("apps").get_object("camera");
if meta.has("last_keyframe") {
//  if time() - 4000 < meta.get_i64("last_keyframe") {
    return meta.get_string("last_jpg");
//  }
}
return DataStore::new().root.join("camera").join("_ASSETS").join("no_signal.jpg").into_os_string().into_string().unwrap();