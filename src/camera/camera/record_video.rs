use ndata::dataobject::*;
use ndata::dataarray::DataArray;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("device");
let a1 = o.get_int("width");
let a2 = o.get_int("height");
let a3 = o.get_int("rot");
let a4 = o.get_int("bitrate");
let a5 = o.get_int("fps");
let ax = record_video(a0, a1, a2, a3, a4, a5);
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn record_video(device:String, width:i64, height:i64, rot:i64, bitrate:i64, fps:i64) -> DataObject {
let mut o = DataArray::new();

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
  if bitrate>0 {
    o.push_string("-b");
    o.push_string(bitrate.to_string());
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
  let mut audiodevice = "default".to_string();
  let mut itsoffset:Option<String> = None;
  let mut video_encoder = "h264".to_string();
  
  if settings.has("audiodevice") { audiodevice = settings.get_string("audiodevice"); }
  if settings.has("itsoffset") { itsoffset = Some(settings.get_string("itsoffset")); }
  if settings.has("video_encoder") { video_encoder = settings.get_string("video_encoder"); }
  
  o.push_string("ffmpeg");
  
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

  if audio {
    o.push_string("-c:a");
    o.push_string("aac");
    o.push_string("-map");
    o.push_string("0:a:0");
    o.push_string("-map");
    o.push_string("1:v:0");
  }
  
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
  
  if settings.has("audio_bitrate") {
    let bs = settings.get_string("audio_bitrate");
    o.push_string("-b:a");
    o.push_string(&bs);
  }
  
  o.push_string(&path);
}

for el in o.objects() {
  print!("{} ", el.string());
}
println!("");



DataObject::new()
}

