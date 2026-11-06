use ndata::dataobject::DataObject;
use ndata::dataarray::DataArray;
use std::env::temp_dir;
use flowlang::flowlang::system::unique_session_id::unique_session_id;
use flowlang::flowlang::system::system_call::system_call;
use std::thread;
use std::fs;
use core::time::Duration;
use crate::camera::camera::available::is_libcamera;
use crate::camera::camera::available::camera_tool;
use crate::camera::camera::available::camera_index;
use flowlang::datastore::DataStore;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["device", "format", "width", "height", "rot"] {
        if !o.has(p) {
            let mut e = DataObject::new();
            e.put_string("status", "err");
            e.put_string("msg", &format!("missing required parameter: {}", p));
            let mut result_obj = DataObject::new();
            result_obj.put_object("a", e);
            return result_obj;
        }
    }
    let ax = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let arg_0: String = o.get_string("device");
        let arg_1: String = o.get_string("format");
        let arg_2: String = o.get_string("width");
        let arg_3: String = o.get_string("height");
        let arg_4: String = o.get_string("rot");
        snapshot(arg_0, arg_1, arg_2, arg_3, arg_4)
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

pub fn snapshot(device: String, format: String, width: String, height: String, rot: String) -> String {
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
}
