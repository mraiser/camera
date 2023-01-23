let mut meta = DataStore::globals().get_object("system").get_object("apps").get_object("camera");
if meta.has("pid") {
  let pid = meta.get_string("pid");
  meta.put_boolean(&pid, false);
}
"OK".to_string()