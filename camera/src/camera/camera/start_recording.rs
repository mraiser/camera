use ndata::dataobject::DataObject;
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
use std::sync::Once;
use std::sync::Mutex;
use crate::camera::camera::available::is_libcamera;
use crate::camera::camera::available::camera_tool;
use crate::camera::camera::available::camera_index;
use crate::camera::camera::available::ffmpeg_input_args;
use crate::camera::camera::available::ffmpeg_encode_args;

pub fn execute(_: DataObject) -> DataObject {
    use std::panic;
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        start_recording()
    }));
    match ax {
        Ok(ax) => {
            let mut result_obj = DataObject::new();
    result_obj.put_string("a", &ax);
            result_obj
        }
        Err(err) => {
            let mut err_obj = DataObject::new();
            err_obj.put_string("status", "err");

            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };

            err_obj.put_string("msg", &msg);
            // Wrapped in the same `a` envelope a successful return uses.
            // Unwrapped, callers that unpack the envelope (newbound's
            // format_result, for one) report an opaque 500 — "Not an object:
            // DString(\"err\")" — instead of this message.
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", err_obj);
            result_obj
        }
    }
}

pub fn start_recording() -> String {
  let guard = CAPTURE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
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
  drop(guard);

  // the capture now retries until the camera opens; stop waiting if this
  // recording is stopped or replaced first
  let beat = Duration::from_millis(10);
  while mp4stream.current_len() == 0 {
    if !meta.has(&pid) || !meta.get_boolean(&pid) { return pid; }
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
      let firsttime = Once::new();
      // has-guard: open_camera's exit path removes the pid key, and this
      // loop must not race a get_boolean against that removal
      while system.get_boolean("running") && meta.has(&tpid) && meta.get_boolean(&tpid) {
        let vec = stream.read(409600);
        if vec.len() > 0 {
          {
            let x1 = writer1.write(&vec);
            if x1.is_err() {  break; }
          }
          if vname1.exists() && vname2.exists() {
            let snow = std::fs::metadata(&vname1).unwrap().created().unwrap();
            let millis:i64 = snow.duration_since(UNIX_EPOCH).expect("error").as_millis().try_into().unwrap();
            //let millis:i64 = start + (vid * 2000);
            
            firsttime.call_once(|| {
              let rounded = millis - (millis % 2000);
              let offset = rounded - meta.get_int("time_1");
              meta.put_int("offset", offset);
            });
            
            let mp4path = to_path(&storage, millis, "archive", "mp4").into_os_string().into_string().unwrap();
            // The jpg does not exist yet: it is the destination that the
            // on_capture handler of the keyframe event writes to; to_path
            // pre-creates the directory so ffmpeg can write there. Event
            // consumers must tolerate a jpg that is still seconds from
            // existing (the UI retries on img error).
            let jpgpath = to_path(&storage, millis, "keyframes", "jpg").into_os_string().into_string().unwrap();
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

// One capture at a time owns the camera: start_recording and start_live
// swap meta's pid/streams under this lock.
static CAPTURE_LOCK: Mutex<()> = Mutex::new(());

// How long a live-only capture (no DVR) stays open with nobody watching:
// long enough to ride out a page reload, short enough to free the camera.
const LIVE_GRACE_MS: i64 = 5000;

// The capture with this pid should keep going: the server runs, and nobody
// set its flag false (stop_recording, a newer start_recording, an idle
// live-only capture, release_idle_capture).
fn capture_wanted(system:&DataObject, meta:&DataObject, tpid:&str) -> bool {
  system.get_boolean("running") && meta.has(tpid) && meta.get_boolean(tpid)
}

// A capture with no consumers at all - no DVR ("mp4") and no live viewer -
// for LIVE_GRACE_MS. A recording always has its "mp4" stream, so it never
// goes idle.
fn live_idle(streams:&DataObject, idle_since:&mut i64) -> bool {
  if streams.objects().len() > 0 { *idle_since = 0; return false; }
  let now = time();
  if *idle_since == 0 { *idle_since = now; }
  now - *idle_since > LIVE_GRACE_MS
}

// A live viewer arrived and nothing is capturing (DVR off): open the
// camera for viewers only - Marc on camera#2: "a live view instead of
// polling snapshots". The capture closes itself LIVE_GRACE_MS after the
// last viewer leaves. False when no camera is chosen yet.
pub fn start_live() -> bool {
  let _g = CAPTURE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
  let mut meta = DataStore::globals().get_object("system").get_object("apps").get_object("camera");
  if meta.has("pid") && meta.has("streams") { return true; }
  let s = settings(Data::DNull);
  let device = if s.has("device") { Data::as_string(s.get_property("device")) } else { "".to_string() };
  if device == "" || device == "null" { return false; }
  let pid = unique_session_id();
  meta.put_string("pid", &pid);
  meta.put_boolean(&pid, true);
  meta.put_object("streams", DataObject::new());
  open_camera(pid);
  true
}

// snapshot needs the camera to itself (the settings preview). A live-only
// capture nobody is watching - the tile pauses its video while the
// settings dialog is open - lets go now rather than after its grace period.
pub fn release_idle_capture() {
  let meta = DataStore::globals().get_object("system").get_object("apps").get_object("camera");
  if !meta.has("pid") || !meta.has("streams") { return; }
  if meta.get_object("streams").objects().len() > 0 { return; }
  let pid = meta.get_string("pid");
  let mut meta = meta;
  meta.put_boolean(&pid, false);
  let mut waited = 0;
  while meta.has(&pid) && waited < 3000 {
    thread::sleep(Duration::from_millis(50));
    waited += 50;
  }
}

fn open_camera(tpid:String) {
  thread::spawn(move || {
    let system = DataStore::globals().get_object("system");
    let mut meta = system.get_object("apps").get_object("camera");
    let mut streams = meta.get_object("streams");

    let settings = settings(Data::DNull);

    let device = settings.get_string("device");
    let format = settings.get_string("format");
    let binding = settings.get_string("resolution");
    // an unset resolution ("null", or never saved) must not panic this
    // thread: fall back to 1280x720
    let res = binding.split("x").collect::<Vec<&str>>();
    let width = res.get(0).and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(1280);
    let height = res.get(1).and_then(|s| s.trim().parse::<i64>().ok()).unwrap_or(720);
    let rot = settings.get_int("rotation");
    let fps = settings.get_int("framerate");
    //let inline_snaps = settings.has("inline_snaps") && Data::as_string(settings.get_property("inline_snaps")) == "true".to_string();

    let mut o = Vec::new();
    let syscmd:String;

    if is_libcamera(&device) {
      syscmd = camera_tool("vid").unwrap_or("libcamera-vid".to_string());
      o.push("--camera".to_owned());
      o.push(camera_index(&device).to_string());

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
      // repeat SPS/PPS with every intra frame: live viewers attach to the
      // fan-out mid-stream and cannot decode without in-band headers (the
      // DVR segmenter is immune - it attaches at stream start and bakes
      // the headers into every segment)
      o.push("--inline".to_owned());
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
    else if let Some(input) = ffmpeg_input_args(std::env::consts::OS, &device, width, height, fps) {
      // C8b: a Mac's or Windows PC's camera hands over raw frames; encode
      // them to the same H.264 stream rpicam emits
      syscmd = "ffmpeg".to_string();
      o.push("-hide_banner".to_string());
      o.extend(input);
      o.extend(ffmpeg_encode_args(fps));
    }
    else {
      syscmd = "ffmpeg".to_string();

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
    
    // One capture feeds the DVR (the "mp4" stream) and every live viewer.
    // A capture nobody asked to stop keeps trying to reopen the camera -
    // it may have been unplugged or powered off and come back (Marc on
    // camera#2: "coming back online after hardware recovery"); the DVR
    // and viewers stay attached to the same streams meanwhile.
    let firsttime = Once::new();
    let mut idle_since:i64 = 0;
    let mut first = true;
    while capture_wanted(&system, &meta, &tpid) {
      if !first {
        println!("Camera stream lost; retrying in 5 s");
        let mut waited = 0;
        while waited < 5000 && capture_wanted(&system, &meta, &tpid) && !live_idle(&streams, &mut idle_since) {
          thread::sleep(Duration::from_millis(250));
          waited += 250;
        }
        if !capture_wanted(&system, &meta, &tpid) || live_idle(&streams, &mut idle_since) { break; }
      }
      first = false;

    let cmd = Command::new(&syscmd)
      .args(&o)
      .stdin(Stdio::null())
      .stderr(Stdio::null())
      .stdout(Stdio::piped())
      .spawn();
    if cmd.is_ok() {
      println!("Camera stream begin {}", time());
      let mut cmd = cmd.unwrap();
      let reader = cmd.stdout.as_mut().unwrap();
      let mut buf = [0; 409600];
      while capture_wanted(&system, &meta, &tpid) {
        let x = reader.read(&mut buf);
        if x.is_err() {  break; }
        let x = x.unwrap();
        if x == 0 { break; }
        
        firsttime.call_once(|| {
          meta.put_int("time_1", time());
        });
        
        let mut dead:Vec<String> = Vec::new();
        for (sid, stream) in streams.objects() {
          let bytes = stream.bytes();
          // write() returns false once a consumer is gone (read side
          // closed); the cap catches one that stopped reading but kept
          // the stream open
          if !bytes.write(&buf[0..x]) || bytes.current_len() > 33554432 {
            dead.push(sid);
          }
        }
        for sid in dead {
          let b = streams.get_bytes(&sid);
          b.close_write();
          streams.remove_property(&sid);
        }
        // a live-only capture whose last viewer left
        if live_idle(&streams, &mut idle_since) { meta.put_boolean(&tpid, false); break; }
      }  

      let _x = cmd.kill();
      // reap it - a killed capture left a defunct ffmpeg per restart
      let _x = cmd.wait();
    }
    }
    for (_id, stream) in streams.objects() {
      stream.bytes().close_write();
    }

    // only our own pid: a newer start_recording has already put its pid
    // here, and removing that one left live viewers seeing "no capture"
    // and stop_recording unable to stop it
    if meta.has("pid") && meta.get_string("pid") == tpid { meta.remove_property("pid"); }
    meta.remove_property(&tpid);
    println!("Camera stream end");
  });
}
