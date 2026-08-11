// lenient: garbage since/limit means 0 (no floor / no cap), never a panic
let since = since.trim().parse::<i64>().unwrap_or(0);
let limit = limit.trim().parse::<i64>().unwrap_or(0);
let settings = settings(Data::DNull);
let root = settings.get_string("storage");
let root = Path::new(&root).join("events");

let a = DataArray::new();
if root.exists() { find(root, a.clone()); }

// newest `limit` events at or after `since` (epoch ms; 0 = no floor),
// returned in ascending time order for direct display
let mut v:Vec<(i64,DataObject)> = Vec::new();
for o in a.objects() {
  let o = o.object();
  let t = if o.has("time") { o.get_int("time") } else { 0 };
  if t >= since { v.push((t, o)); }
}
v.sort_by_key(|x| x.0);
let n = v.len();
let skip = if limit > 0 && n > limit as usize { n - limit as usize } else { 0 };
let mut out = DataArray::new();
for (_t, o) in v.into_iter().skip(skip) { out.push_object(o); }
out
}

fn find(dir:PathBuf, mut list:DataArray) {
  if let Ok(rd) = fs::read_dir(dir) {
    for f in rd {
      if f.is_err() { continue; }
      let f = f.unwrap();
      let meta = match fs::metadata(f.path()) { Ok(m) => m, Err(_) => continue };
      if meta.is_dir() {
        find(f.path(), list.clone());
      }
      else {
        let name = match f.file_name().into_string() { Ok(n) => n, Err(_) => continue };
        if name.ends_with(".json") {
          if let Ok(s) = fs::read_to_string(f.path()) {
            let o = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| DataObject::from_string(&s)));
            if let Ok(o) = o { list.push_object(o); }
          }
        }
      }
    }
  }