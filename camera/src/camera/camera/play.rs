use ndata::dataobject::DataObject;
use crate::camera::camera::settings::settings;
use ndata::data::Data;
use chrono::DateTime;
use chrono::Utc;
use std::path::Path;
use chrono::TimeZone;
use chrono::Datelike;
use chrono::Timelike;
use std::path::PathBuf;
use std::env::temp_dir;
use flowlang::flowlang::system::unique_session_id::unique_session_id;
use std::fs::File;
use std::io::Write;
use ndata::dataarray::DataArray;
use std::fs;
use flowlang::flowlang::system::system_call::system_call;
use crate::camera::camera::available::is_libcamera;

pub fn execute(o: DataObject) -> DataObject {
    use std::panic;
    for p in ["nn_path"] {
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
        let arg_0: String = o.get_string("nn_path");
        play(arg_0)
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

pub fn play(nn_path: String) -> String {
  let settings = settings(Data::DNull);
  let storage = settings.get_string("storage");
  let format = settings.get_string("format");
  let mut x = nn_path[13..].split(".");
  let mut x = x.next().unwrap().split("_");
  let start = x.next().unwrap().parse::<i64>().unwrap();
  let stop = x.next().unwrap().parse::<i64>().unwrap();
  let dest = Path::new(&storage).join("generated");
  let _x = fs::create_dir_all(&dest).unwrap();
  let generated = dest.join(start.to_string()+"_"+&stop.to_string()+".mp4");
  if generated.exists() { return generated.into_os_string().into_string().unwrap(); }

  let dir = temp_dir();
  let dest = unique_session_id();
  let filename = dest.to_owned()+".txt";
  let chunk_list = dir.join(filename);
  let mut b = false;
  {
    let mut f = File::create(chunk_list.to_owned()).expect("Unable to create file");
    let mut time = start;
    while time <= stop {
      let chunkf = to_path(&storage, time);
      if chunkf.exists() {
        let s = fs::canonicalize(chunkf).unwrap().into_os_string().into_string().unwrap();
        let s = "file ".to_string()+&s+"\n";
        let _x = f.write(&s.as_bytes()).unwrap();
        b = true;
      }
      time += 2000;
    }
  }

  if b {
    let rot = settings.get_int("rotation");
    let mut o = DataArray::new();
    o.push_string("ffmpeg");
    o.push_string("-y");
    o.push_string("-f");
    o.push_string("concat");
    o.push_string("-safe");
    o.push_string("0");
    
    if rot != 0 && is_libcamera(&settings.get_string("device")) {
      o.push_string("-display_rotation");
      o.push_string(&(360-rot).to_string());
    }
    
    o.push_string("-i");
    o.push_string(&chunk_list.to_owned().into_os_string().into_string().unwrap());
    if format == "MJPG".to_string() {
      o.push_string("-pix_fmt");
      o.push_string("yuv420p");
      o.push_string("-c:v");
      o.push_string("h264");
    }
    else if rot != 0 && !is_libcamera(&settings.get_string("device")) {
      let t = match rot {
        90 => "transpose=1",
        180 => "transpose=2,transpose=2",
        270 => "transpose=2",
        _ => "transpose=none",
      };
      let mut t = t.to_string();
      if rot == 90 || rot == 270 {
        let binding = settings.get_string("resolution");
        let res = binding.split("x").collect::<Vec<&str>>();
        let width = res[0].parse::<i64>().unwrap();
        let height = res[1].parse::<i64>().unwrap();
        let q = (width*width)/height;
        t = t + ",scale=" + &width.to_string()+":"+&q.to_string()+",crop="+&width.to_string()+":"+&height.to_string();
      }
      o.push_string("-filter:v");
      o.push_string(&t);
    }
    else {
      o.push_string("-c:v");
      o.push_string("copy");
    }

    o.push_string("-movflags");
    o.push_string("faststart");
    
    let vidfile = generated.to_owned();
    o.push_string(&vidfile.to_owned().into_os_string().into_string().unwrap());
    let _x = system_call(o);
  }

  let _x = fs::remove_file(chunk_list).unwrap();
  generated.into_os_string().into_string().unwrap()
}

fn to_path(storage:&str, time:i64) -> PathBuf {
  let now:DateTime<Utc> = Utc.timestamp_millis_opt(time).unwrap();
  let year = now.year();
  let month = now.month();
  let day = now.day();
  let hour = now.hour();
  let minute = now.minute();
  let second = now.second();
  let index = second / 2;
  let dir = Path::new(&storage).join("archive").join(year.to_string()).join(month.to_string()).join(day.to_string()).join(hour.to_string()).join(minute.to_string());
  let fname = index.to_string()+".mp4";
  dir.join(fname)
}
