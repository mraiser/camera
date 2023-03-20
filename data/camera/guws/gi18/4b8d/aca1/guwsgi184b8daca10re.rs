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
    
    if rot != 0 && settings.get_string("device") == "libcamera-apps".to_string() {
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
    else if rot != 0 && settings.get_string("device") != "libcamera-apps".to_string() {
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