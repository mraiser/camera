let mut o = DataArray::new();

let settings = settings(Data::DNull);

let device = settings.get_string("device");
let format = settings.get_string("format");
let binding = settings.get_string("resolution");
let res = binding.split("x").collect::<Vec<&str>>();
let width = res[0].parse::<i64>().unwrap();
let height = res[1].parse::<i64>().unwrap();
let rot = settings.get_int("rotation");
let fps = settings.get_int("framerate");
let storage = settings.get_string("storage");
let dest = Path::new(&storage).join("capture");
let path = dest.to_owned().join("%d.mp4").into_os_string().into_string().unwrap();

let _x = fs::remove_dir_all(&dest);
let _x = fs::create_dir_all(&dest);

let mut audio = false;
if settings.has("audio") && settings.get_string("audio") == "true".to_string() {
  audio = true;
  let settingsx = settings.clone();
  let destx = dest.to_owned();
  thread::spawn(move || {
    let settings = settingsx;
    let beat = Duration::from_millis(1000);
    thread::sleep(beat);

    let mut audiodevice = "default".to_string();
    if settings.has("audiodevice") { audiodevice = settings.get_string("audiodevice"); }

    let mut o = DataArray::new();
    o.push_string("-f");
    o.push_string("alsa");
    o.push_string("-ac");
    o.push_string("2");
    o.push_string("-i");
    o.push_string(&audiodevice);
    o.push_string("-c:a");
    o.push_string("aac");
    o.push_string("-segment_time");
    o.push_string("2");
    o.push_string("-f");
    o.push_string("segment");

    if settings.has("audio_bitrate") {
      let bs = settings.get_string("audio_bitrate");
      o.push_string("-b:a");
      o.push_string(&bs);
    }

    o.push_string(&destx.join("%d.aac").into_os_string().into_string().unwrap());
    
    print!("ffmpeg ");
    for el in o.objects() {
      print!("{} ", el.string());
    }
    println!("");
    
    let mut args = Vec::<String>::new();
    for arg in o.objects() {
      args.push(arg.string());
    }

    let _cmd = Command::new("ffmpeg")
      .args(args)
      .stderr(Stdio::null())
      .stdout(Stdio::null())
      .spawn()
      .expect("failed to execute process");
  });    
}

if device == "libcamera-apps".to_string() {
  let mut width = width;
  let mut height = height;
  if rot == 90 || rot == 270 {
    let w = width;
    width = height;
    height = w;
  }

  o.push_string("libcamera-vid");
/*  
  if audio {
    o.push_string("--codec");
    o.push_string("libav");
    o.push_string("--libav-audio");
    if settings.has("audio_bitrate") {
      let bs = settings.get_string("audio_bitrate");
      o.push_string(&("--audio-bitrate=".to_string()+&bs));
    }
  }
*/  
  if settings.has("bitrate") {
    let bs = settings.get_string("bitrate");
    o.push_string("-b");
    o.push_string(&bs);
  }
  
  o.push_string("--framerate");
  o.push_string(&fps.to_string());
  o.push_string("--segment");
  o.push_string("1000");
  o.push_string("--nopreview");
  o.push_string("--inline");
  o.push_string("-t");
  o.push_string("0");
  o.push_string("--width");
  o.push_string(&width.to_string());
  o.push_string("--height");
  o.push_string(&height.to_string());
  o.push_string("-o");
  o.push_string(&path);  
}
else {
//  let mut audiodevice = "default".to_string();
//  let mut itsoffset:Option<String> = None;
  let mut video_encoder = "h264".to_string();
  
//  if settings.has("audiodevice") { audiodevice = settings.get_string("audiodevice"); }
//  if settings.has("itsoffset") { itsoffset = Some(settings.get_string("itsoffset")); }
  if settings.has("video_encoder") { video_encoder = settings.get_string("video_encoder"); }
  
  o.push_string("ffmpeg");
/*  
  if audio {
    o.push_string("-f");
    o.push_string("alsa");
    o.push_string("-ac");
    o.push_string("2");
    o.push_string("-i");
    o.push_string(&audiodevice);
    if itsoffset.is_some() {
      o.push_string("-itsoffset");
      o.push_string(&itsoffset.unwrap());
    }
  }
*/  
  o.push_string("-f");
  o.push_string("v4l2");
  
  if format == "H264".to_string() { o.push_string("-vcodec"); o.push_string("h264"); }
  else if format == "MJPG".to_string() { o.push_string("-vcodec"); o.push_string("mjpeg"); }
  else if format == "YUYV".to_string() { o.push_string("-vcodec"); o.push_string("yuyv422"); }
  // FIXME - What are all the others?

  o.push_string("-framerate");
  o.push_string(&fps.to_string());
  o.push_string("-video_size");
  o.push_string(&(width.to_string()+"x"+&height.to_string()));
  o.push_string("-i");
  o.push_string(&device);

  o.push_string("-c:v");
  o.push_string(&video_encoder); 
/*
  if audio {
    o.push_string("-c:a");
    o.push_string("aac");
    o.push_string("-map");
    o.push_string("0:a:0");
    o.push_string("-map");
    o.push_string("1:v:0");
  }
*/  
  o.push_string("-segment_time");
  o.push_string("2");
  o.push_string("-g");
  o.push_string("2");
  o.push_string("-sc_threshold");
  o.push_string("0");
  o.push_string("-force_key_frames");
  o.push_string("expr:gte(t,n_forced*2)");
  o.push_string("-f");
  o.push_string("segment");
  
  if settings.has("bitrate") {
    let bs = settings.get_string("bitrate");
    o.push_string("-b:v");
    o.push_string(&bs);
  }
/*  
  if settings.has("audio_bitrate") {
    let bs = settings.get_string("audio_bitrate");
    o.push_string("-b:a");
    o.push_string(&bs);
  }
*/  
  o.push_string(&path);
}

for el in o.objects() {
  print!("{} ", el.string());
}
println!("");

let system = DataStore::globals().get_object("system");
let mut meta = system.get_object("apps").get_object("camera");
let pid = unique_session_id();
meta.put_string("pid", &pid);
meta.put_boolean(&pid, true);

let mut command = o.clone();
let tpid = pid.to_owned();
thread::spawn(move || {
  let beat = Duration::from_millis(100);
  
  let a = command.get_string(0);
  command.remove_property(0);

  let mut args = Vec::<String>::new();
  for arg in command.objects() {
    args.push(arg.string());
  }

  let mut cmd = Command::new(&a)
    .args(args)
    .stderr(Stdio::null())
    .stdout(Stdio::null())
    .spawn()
    .expect("failed to execute process");
  
  let archive = Path::new(&storage).join("archive");
  let mut i = 0;
  let mut name1 = dest.join(i.to_string()+".mp4");
  let mut name2 = dest.join((i+1).to_string()+".mp4");
  let mut name3 = dest.join((i+2).to_string()+".mp4");
  while system.get_boolean("running") && meta.get_boolean(&tpid) {
    thread::sleep(beat);
    
    if name1.exists() && name2.exists() {
      let snow = std::fs::metadata(&name1).unwrap().modified().unwrap();
      let now:DateTime<Utc> = snow.into();
      let year = now.year();
      let month = now.month();
      let day = now.day();
      let hour = now.hour();
      let minute = now.minute();
      let second = now.second();
      let index = second / 2;

      let f2 = archive.join(year.to_string()).join(month.to_string()).join(day.to_string()).join(hour.to_string()).join(minute.to_string());
      let _x = fs::create_dir_all(&f2);
      let f2 = f2.join(index.to_string()+".mp4");
      let _x = fs::rename(name1, &f2);
      
      if audio {
        let namea = dest.join(i.to_string()+".aac");
        let f2 = archive.join(year.to_string()).join(month.to_string()).join(day.to_string()).join(hour.to_string()).join(minute.to_string());
        let _x = fs::create_dir_all(&f2);
        let mut suffix = "".to_string();
        while (&f2).join(index.to_string()+&suffix+".aac").exists() {
          suffix += "_X";
        }
        let f2 = f2.join(index.to_string()+&suffix+".aac");
        let _x = fs::rename(namea, &f2);
      }
      
      let millis:i64 = snow.duration_since(UNIX_EPOCH).expect("error").as_millis().try_into().unwrap();
//      let mut x = DataObject::new();
//      x.put_string("path", &f2.into_os_string().into_string().unwrap());
//      x.put_int("timestamp", millis);
//      fire_event("camera", "capture", x);
      if !name3.exists() {
//        println!("capture {:?}", f2);
        on_capture(f2.into_os_string().into_string().unwrap(), millis);
      }
//      else {
//        println!("move {:?}", f2);
//      }
      
      i += 1;
      name1 = name2;
      name2 = name3;
      name3 = dest.join((i+2).to_string()+".mp4");
    }
  }
  
  let _x = cmd.kill();
  meta.remove_property("pid");
  meta.remove_property(&tpid);
  
  println!("Video capture done.");
});

pid
