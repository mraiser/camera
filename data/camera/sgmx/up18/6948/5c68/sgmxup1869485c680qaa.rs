let system = DataStore::globals().get_object("system");
let meta = system.get_object("apps").get_object("camera");

// No capture running (DVR off): the platform 404 sentinel
if !meta.has("pid") || !meta.has("streams") {
  return DataBytes::from_bytes(&"404".as_bytes().to_vec());
}

let mut streams = meta.get_object("streams");

let inputstream = DataBytes::new();
let id = unique_session_id();
streams.put_bytes(&id, inputstream.clone());

let outputstream = DataBytes::new();
outputstream.set_mime_type(Some("video/mp4".to_string()));
let o2 = outputstream.clone();

thread::spawn(move || {
  let settings = settings(Data::DNull);
  let fps = settings.get_int("framerate");
  let device = settings.get_string("device");
  let format = settings.get_string("format");
  let video_encoder = match settings.has("video_encoder") {
    true => settings.get_string("video_encoder"),
    _ => "copy".to_string()
  };

  // The fan-out carries whatever the capture process emits: an H.264
  // elementary stream for libcamera, v4l2-H264, or an h264 encoder;
  // MJPEG when an MJPG source is passed through. H.264 remuxes with
  // -c copy (no transcode on the Pi); anything else must be encoded
  // for the browser.
  let h264 = is_libcamera(&device) || format == "H264".to_string() || video_encoder == "h264".to_string();

  let mut args:Vec<String> = Vec::new();
  args.push("-framerate".to_string()); args.push(fps.to_string());
  if h264 { args.push("-f".to_string()); args.push("h264".to_string()); }
  else if format == "MJPG".to_string() { args.push("-f".to_string()); args.push("mjpeg".to_string()); }
  args.push("-i".to_string()); args.push("-".to_string());
  if h264 {
    args.push("-c:v".to_string()); args.push("copy".to_string());
  }
  else {
    args.push("-c:v".to_string()); args.push("libx264".to_string());
    args.push("-preset".to_string()); args.push("ultrafast".to_string());
    args.push("-tune".to_string()); args.push("zerolatency".to_string());
    args.push("-pix_fmt".to_string()); args.push("yuv420p".to_string());
  }
  args.push("-movflags".to_string()); args.push("frag_keyframe+empty_moov+default_base_moof".to_string());
  args.push("-f".to_string()); args.push("mp4".to_string());
  args.push("-".to_string());

  print!("{}", "ffmpeg"); for el in &args { print!(" {}", el); } println!("");

  let cmd = Command::new("ffmpeg")
    .args(args)
    .stdin(Stdio::piped())
    .stderr(Stdio::null())
    .stdout(Stdio::piped())
    .spawn();
  if cmd.is_err() {
    streams.remove_property(&id);
    inputstream.close_read();
    o2.close_write();
    return;
  }
  let mut child = cmd.unwrap();
  let mut stdin = child.stdin.take().unwrap();
  let mut stdout = child.stdout.take().unwrap();

  // ffmpeg stdout -> HTTP consumer. When the appserver loses the socket
  // it close_read()s the response DataBytes and write() returns false -
  // that is the teardown signal. A stalled-but-open client is caught by
  // the buffer cap instead.
  let pump = thread::spawn(move || {
    let mut buf = [0; 409600];
    loop {
      let x = stdout.read(&mut buf);
      if x.is_err() { break; }
      let x = x.unwrap();
      if x == 0 { break; }
      if !o2.write(&buf[0..x]) { break; }
      if o2.current_len() > 33554432 { break; }
    }
    o2.close_write();
  });

  // capture fan-out -> ffmpeg stdin. When the pump ends it drops ffmpeg
  // stdout, ffmpeg dies on the broken pipe, and this write fails.
  println!("Live viewer connected");
  let beat = Duration::from_millis(10);
  while inputstream.is_read_open() {
    let vec = inputstream.read(409600);
    if vec.len() > 0 {
      if stdin.write_all(&vec).is_err() { break; }
    }
    else {
      if pump.is_finished() { break; }
      thread::sleep(beat);
    }
  }

  let _x = child.kill();
  let _x = child.wait();
  let _x = pump.join();

  // deregister from the capture fan-out - the pre-MSE version leaked one
  // dead consumer per view
  streams.remove_property(&id);
  inputstream.close_read();
  println!("Live viewer disconnected");
});

outputstream