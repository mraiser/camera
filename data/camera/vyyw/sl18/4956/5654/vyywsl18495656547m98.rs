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
