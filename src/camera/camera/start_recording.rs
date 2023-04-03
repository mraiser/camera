use ndata::dataobject::*;
use std::thread;
use flowlang::datastore::DataStore;
use std::process::Command;
use std::process::Stdio;
use std::path::Path;
use std::fs::remove_dir_all;
use std::fs::create_dir_all;
use std::fs::rename;
use crate::camera::camera::settings::settings;
use ndata::data::Data;
use flowlang::flowlang::system::unique_session_id::unique_session_id;
use std::io::Read;
use std::io::Write;
use std::time::UNIX_EPOCH;
use flowlang::appserver::fire_event;
use ndata::databytes::DataBytes;
use core::time::Duration;
use crate::camera::camera::on_capture::to_path;
use flowlang::flowlang::system::time::time;

pub fn execute(_o: DataObject) -> DataObject {
let ax = start_recording();
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn start_recording() -> String {
  let mut meta = DataStore::globals().get_object("system").get_object("apps").get_object("camera");
  if meta.has("pid") { meta.put_boolean(&meta.get_string("pid"), false); }
  let pid = unique_session_id();
  meta.put_string("pid", &pid);
  meta.put_boolean(&pid, true);
  
  let mut streams = DataObject::new();
  meta.put_object("streams", streams.clone());

  let mp4stream = DataBytes::new();
  streams.put_bytes("mp4", mp4stream.clone());
  
  open_camera(pid.to_owned());

  let beat = Duration::from_millis(10);
  while mp4stream.current_len() == 0 {
    thread::sleep(beat);
  }

  write_video(mp4stream, pid.to_owned());

  pid
}

fn write_video(stream:DataBytes, tpid:String){
  thread::spawn(move || {
    let system = DataStore::globals().get_object("system");
    let mut meta = system.get_object("apps").get_object("camera");
    let settings = settings(Data::DNull);
    let storage = settings.get_string("storage");
    let dest = Path::new(&storage).join("capture");
    let _x = remove_dir_all(&dest);
    let _x = create_dir_all(&dest);
    let fps = settings.get_int("framerate");
	let path1 = dest.to_owned().join("%d.mp4").into_os_string().into_string().unwrap();
  
    let args1 = ["-framerate", &fps.to_string(), "-i", "-", "-segment_time", "2", "-g", "2", "-sc_threshold", "0", "-force_key_frames", "expr:gte(t,n_forced*2)", "-c", "copy", "-f", "segment", &path1];
    let args1 = args1.to_vec();
    print!("{}", "ffmpeg"); for el in &args1 { print!(" {}", el); } println!("");
    let cmd1 = Command::new("ffmpeg")
      .args(args1)
      .stdin(Stdio::piped())
      .stderr(Stdio::null())
      .stdout(Stdio::null())
      .spawn();
    if cmd1.is_ok() {
      println!("Video write begin");
      let mut cmd1 = cmd1.unwrap();
      let writer1 = cmd1.stdin.as_mut().unwrap();
      let mut vid = 0;
      let mut vname1 = dest.join(vid.to_string()+".mp4");
      let mut vname2 = dest.join((vid+1).to_string()+".mp4");
      let beat = Duration::from_millis(10);
      //let start = time();
      while system.get_boolean("running") && (&meta).get_boolean(&tpid) {
        let vec = stream.read(409600);
        if vec.len() > 0 {
          {
            let x1 = writer1.write(&vec);
            if x1.is_err() {  break; }
          }
          if vname1.exists() && vname2.exists() {
            let snow = std::fs::metadata(&vname1).unwrap().modified().unwrap();
            let millis:i64 = snow.duration_since(UNIX_EPOCH).expect("error").as_millis().try_into().unwrap();
            //let millis:i64 = start + (vid * 2000);
            let mp4path = to_path(&storage, millis, "archive", "mp4").into_os_string().into_string().unwrap();
            let jpgpath = to_path(&storage, millis, "keyframes", "jpg").into_os_string().into_string().unwrap(); // FIXME - No guarantee this exists. Expensive to do twice in a row like that
            let _x = rename(&vname1, &mp4path);
            meta.put_string("last_mp4", &mp4path);
            meta.put_int("last_keyframe", millis);

            let plen = storage.len();
            let rpath = &mp4path[plen..];
            let rpath2 = &jpgpath[plen..];

            let mut x = DataObject::new();
            x.put_string("mp4", rpath);
            x.put_string("jpg", rpath2);
            x.put_int("timestamp", millis);
            fire_event("camera", "keyframe", x);

            vid += 1;
            vname1 = vname2;
            vname2 = dest.join((vid+1).to_string()+".mp4");
          }
        }
        else {
          thread::sleep(beat);
        }
      }
      let _x = cmd1.kill();
      println!("Video write end");
    }
  });
}

fn open_camera(tpid:String) {
  thread::spawn(move || {
    let system = DataStore::globals().get_object("system");
    let mut meta = system.get_object("apps").get_object("camera");
    let streams = meta.get_object("streams");

    let settings = settings(Data::DNull);

    let device = settings.get_string("device");
    let format = settings.get_string("format");
    let binding = settings.get_string("resolution");
    let res = binding.split("x").collect::<Vec<&str>>();
    let width = res[0].parse::<i64>().unwrap();
    let height = res[1].parse::<i64>().unwrap();
    let rot = settings.get_int("rotation");
    let fps = settings.get_int("framerate");
    //let inline_snaps = settings.has("inline_snaps") && Data::as_string(settings.get_property("inline_snaps")) == "true".to_string();

    let mut o = Vec::new();
    let syscmd;

    if device == "libcamera-apps".to_string() {
      syscmd = "libcamera-vid";

      let mut width = width;
      let mut height = height;
      if rot == 90 || rot == 270 {
        let w = width;
        width = height;
        height = w;
      }

      if settings.has("bitrate") {
        let bs = settings.get_string("bitrate");
        o.push("-b".to_owned());
        o.push(bs.to_owned());
      }

      o.push("--framerate".to_owned());
      o.push(fps.to_string());
      o.push("--nopreview".to_owned());
      o.push("-t".to_owned());
      o.push("0".to_owned());
      o.push("--width".to_owned());
      o.push(width.to_string());
      o.push("--height".to_owned());
      o.push(height.to_string());
      o.push("-o".to_owned());
      o.push("-".to_owned());  
    }
    else {
      syscmd = "ffmpeg";

      o.push("-f".to_string());
      o.push("v4l2".to_string());

      let vcodec = match format.as_ref() {
        "H264" => "h264",
        "MJPG" => "mjpeg",
        "YUYV" => "yuyv422",
        _ => "h264" // FIXME - What are the others?
      };
      o.push("-vcodec".to_string()); 
      o.push(vcodec.to_string());

      o.push("-framerate".to_string());
      o.push(fps.to_string());
      o.push("-video_size".to_string());
      o.push(width.to_string()+"x"+&height.to_string());
      o.push("-i".to_string());
      o.push(device.to_owned());

      if settings.has("bitrate") {
        let bs = settings.get_string("bitrate");
        o.push("-b:v".to_owned());
        o.push(bs.to_owned());
      }
      
      let video_encoder = match settings.has("video_encoder") {
        true => settings.get_string("video_encoder"),
        _ => "copy".to_string()
      };
      
      o.push("-c".to_string());
      o.push(video_encoder);

      o.push("-f".to_string());
      o.push(vcodec.to_string());
      o.push("-".to_string());
    }

    print!("{}", syscmd); for el in &o { print!(" {}", el); } println!("");

//    let when = time();
//    let when = when - (when % 2000) + 2000;
//    let beat = Duration::from_millis(1);
//    while time() < when { 
//      thread::sleep(beat);
//    }
//    println!("AND THE VIDEO TIME IS {}", time());
    
    let cmd = Command::new(syscmd)
      .args(o)
      .stderr(Stdio::null())
      .stdout(Stdio::piped())
      .spawn();
    if cmd.is_ok() {
      println!("Camera stream begin {}", time());
      let mut cmd = cmd.unwrap();
      let reader = cmd.stdout.as_mut().unwrap();
      let mut buf = [0; 409600];
      while system.get_boolean("running") && meta.get_boolean(&tpid) {
        let x = reader.read(&mut buf);
        if x.is_err() {  break; }
        let x = x.unwrap();
        if x == 0 { break; }
        for (_id, stream) in streams.objects() {
          stream.bytes().write(&buf[0..x]);
        }
      }  

      let _x = cmd.kill();
      for (_id, stream) in streams.objects() {
        stream.bytes().close_write();
      }
    }

    meta.remove_property("pid");
    meta.remove_property(&tpid);
    println!("Camera stream end");
  });
}

