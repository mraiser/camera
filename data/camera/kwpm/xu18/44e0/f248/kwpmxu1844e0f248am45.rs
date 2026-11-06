let system = DataStore::globals().get_object("system");
let meta = system.get_object("apps").get_object("camera");
if meta.has("last_jpg") {
  // C7: a keyframe is only "live" while frames are landing - one older than
  // 10s means the picture stopped, and the caller gets the no-signal card
  if meta.has("last_keyframe") && time() - 10000 < meta.get_int("last_keyframe") {
    return meta.get_string("last_jpg");
  }
}
return DataStore::new().root.join("camera").join("_ASSETS").join("no_signal.jpg").into_os_string().into_string().unwrap();