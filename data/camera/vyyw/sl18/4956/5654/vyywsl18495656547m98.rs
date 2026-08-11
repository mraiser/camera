let settings = settings(Data::DNull);
let root = settings.get_string("storage");
let root = Path::new(&root).join("events");

let a = DataArray::new();
if root.exists() { find(root, a.clone()); }

// server-side sort by event time, ascending (was left to the client)
let mut v:Vec<(i64,DataObject)> = Vec::new();
for o in a.objects() {
  let o = o.object();
  let t = if o.has("time") { o.get_int("time") } else { 0 };
  v.push((t, o));
}
v.sort_by_key(|x| x.0);
let mut out = DataArray::new();
for (_t, o) in v { out.push_object(o); }
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
            // tolerate a torn write (crash mid-motion_end): skip a file
            // that does not parse instead of erroring the whole listing
            let o = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| DataObject::from_string(&s)));
            if let Ok(o) = o { list.push_object(o); }
          }
        }
      }
    }
  }