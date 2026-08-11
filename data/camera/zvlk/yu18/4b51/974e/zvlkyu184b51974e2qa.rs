let settings = settings(Data::DNull);
let storage = settings.get_string("storage");
// nn_path comes straight off the URL: serve only from inside the
// keyframes tree, never a traversal out of it
if nn_path.len() < 18 || nn_path.contains("..") {
  return DataStore::new().root.join("camera").join("_ASSETS").join("no_signal.jpg").into_os_string().into_string().unwrap();
}
let x = storage + "/keyframes/" + &nn_path[18..];
x