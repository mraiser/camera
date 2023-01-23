let settings = settings(Data::DNull);
if settings.has("dvr") && settings.get_boolean("dvr"){
  thread::spawn(move || {
    start_recording();
  });
  thread::spawn(move || {
    let system = DataStore::globals().get_object("system");
    let beat = Duration::from_millis(60000);
    while system.get_boolean("running") {
      remove_old();
      thread::sleep(beat);
    }
  });
}

let store = DataStore::new();
let mut d;
if store.exists("runtime", "controls_available") { d = store.get_data("runtime", "controls_available").get_object("data"); }
else { d = DataObject::new(); }

let mut b = false;
if !d.has("camera:dvr") {
  let o = DataObject::from_string("{\"title\":\"Camera/DVR\",\"type\":\"camera:dvr\",\"big\":true,\"position\":\"inline\",\"groups\":[\"admin\"]}");
  d.put_object("camera:dvr", o);
  b = true;
}

if b { write("runtime".to_string(), "controls_available".to_string(), d.clone(), DataArray::new(), DataArray::new()); }

"OK".to_string()