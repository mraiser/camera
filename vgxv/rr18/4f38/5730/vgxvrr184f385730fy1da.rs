let settings = settings(Data::DNull);
let storage = settings.get_string("storage");
let path = Path::new(&storage);
let days;
if settings.has("days") { days = settings.get_string("days").parse::<f64>().unwrap(); }
else { days = 2.0; }
let millis = (days * 24.0 * 60.0 * 60.0 * 1000.0) as i64;
let time = time() - millis;

cull(&path.join("archive"), time);
cull(&path.join("events"), time);
cull(&path.join("generated"), time);
cull(&path.join("keyframes"), time);

"OK".to_string()
}

fn cull(path:&Path, time:i64) {
  if path.exists() {
    for file in fs::read_dir(&path).unwrap() {
      let file = file.unwrap();
      let path = file.path();
      let metadata = fs::metadata(path.to_owned()).unwrap();
      if metadata.is_dir() {
        cull(&path, time);
      }
      else if metadata.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis() < time as u128 {
        let _x = fs::remove_file(&path);
      }
    }
    if path.read_dir().unwrap().next().is_none() {
      let _x = fs::remove_dir_all(&path);
    }
  }