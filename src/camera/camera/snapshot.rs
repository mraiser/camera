use ndata::dataobject::*;
use ndata::dataarray::DataArray;
use std::env::temp_dir;
use flowlang::flowlang::system::unique_session_id::unique_session_id;
use flowlang::flowlang::system::system_call::system_call;
use std::thread;
use std::fs;
use core::time::Duration;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("device");
let a1 = o.get_string("format");
let a2 = o.get_int("width");
let a3 = o.get_int("height");
let a4 = o.get_int("rot");
let ax = snapshot(a0, a1, a2, a3, a4);
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn snapshot(device:String, format:String, width:i64, height:i64, rot:i64) -> String {
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

if device == "libcamera-apps".to_string(){
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
  o.push_string("libcamera-still");
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


thread::spawn(move || {
  thread::sleep(Duration::from_millis(10000));
  let _x = fs::remove_file(t1);
});

temp
}

