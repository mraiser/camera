// URL-facing params arrive as raw strings and must never panic the
// handler thread (flowlang cast_params aborts on a malformed Integer,
// and the settings preview retries forever): parse leniently and
// soft-fail to the no-signal placeholder.
let width = width.trim().parse::<i64>().unwrap_or(0);
let height = height.trim().parse::<i64>().unwrap_or(0);
let rot = rot.trim().parse::<i64>().unwrap_or(0);
if width <= 0 || height <= 0 {
  return DataStore::new().root.join("camera").join("_ASSETS").join("no_signal.jpg").into_os_string().into_string().unwrap();
}
let t1 = temp_dir().join(unique_session_id()+".jpg");
let temp = t1.to_owned().into_os_string().into_string().unwrap();
//println!("temp file {:?}", temp);
//println!("device {}", device);

let t = match rot {
  90 => "transpose=1",
  180 => "transpose=2,transpose=2",
  270 => "transpose=2",
  _ => "transpose=none",
};

if is_libcamera(&device) {
  let mut width = width;
  let mut height = height;
  if rot == 90 || rot == 270 {
    let w = width;
    width = height;
    height = w;
  }
  
  let t2 = temp_dir().join(unique_session_id()+"_x.jpg");
  let temp2 = t2.to_owned().into_os_string().into_string().unwrap();
  let mut o = DataArray::new();
  o.push_string(&camera_tool("still").unwrap_or("libcamera-still".to_string()));
  o.push_string("--camera");
  o.push_string(&camera_index(&device).to_string());
  o.push_string("-n");
  o.push_string("--width");
  o.push_string(&width.to_string());
  o.push_string("--height");
  o.push_string(&height.to_string());
  o.push_string("-o");
  o.push_string(&temp2);
//  println!("{}", o.to_string());
  let _x = system_call(o);

  let mut o = DataArray::new();
  o.push_string("ffmpeg");
  o.push_string("-i");
  o.push_string(&temp2);
  if rot != 0 {
    o.push_string("-filter:v");
    o.push_string(&t);
  }
  o.push_string(&temp);
//  println!("{}", o.to_string());
  let _x = system_call(o);
  let _x = fs::remove_file(t2);
}
else {
  // device comes straight off the URL; V4L2 capture devices live under
  // /dev - refuse anything else (ffmpeg -i would happily read any path)
  if !device.starts_with("/dev/") { panic!("snapshot failed: '{}' is not a camera device on this machine", device); }
  let mut t = t.to_string();
  if rot == 90 || rot == 270 { // FIXME - If camera supports portrait scaling degrades the image unnecessarily
    let q = (width*width)/height;
    t = t + ",scale=" + &width.to_string()+":"+&q.to_string()+",crop="+&width.to_string()+":"+&height.to_string();
  }
  
  let mut o = DataArray::new();
  o.push_string("ffmpeg");
  o.push_string("-f");
  o.push_string("v4l2");
  if format == "H264".to_string() { o.push_string("-vcodec"); o.push_string("h264"); }
  else if format == "MJPG".to_string() { o.push_string("-vcodec"); o.push_string("mjpeg"); }
  else if format == "YUYV".to_string() { o.push_string("-vcodec"); o.push_string("yuyv422"); }
  // FIXME - What are all the others?
  o.push_string("-video_size");
  o.push_string(&(width.to_string()+"x"+&height.to_string()));
  o.push_string("-i");
  o.push_string(&device);
  o.push_string("-update");
  o.push_string("1");
  o.push_string("-vframes:v");
  o.push_string("1");
  if rot != 0 {
    o.push_string("-filter:v");
    o.push_string(&t);
  }
  o.push_string(&temp);
  let _x = system_call(o);
}


// C7: a capture that produced no image is an error, never a path to a file
// that was never written
if !t1.exists() {
  panic!("snapshot failed: the capture tool wrote no image for device '{}' - the camera may be disconnected or its capture tool (rpicam-still / ffmpeg) missing", device);
}

thread::spawn(move || {
  thread::sleep(Duration::from_millis(10000));
  let _x = fs::remove_file(t1);
});

temp