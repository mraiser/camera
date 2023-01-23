  let settings = settings(Data::DNull);
  let storage = settings.get_string("storage");
  let mut x = nn_path[13..].split(".");
  let mut x = x.next().unwrap().split("_");
  let start = x.next().unwrap().parse::<i64>().unwrap();
  let stop = x.next().unwrap().parse::<i64>().unwrap();
  let dest = Path::new(&storage).join("generated");
  let _x = fs::create_dir_all(&dest).unwrap();
  let generated = dest.join(start.to_string()+"_"+&stop.to_string()+".mp4");
  if generated.exists() { return generated.into_os_string().into_string().unwrap(); }

  let mut audio = false;
  if settings.has("audio") && settings.get_string("audio") == "true".to_string() {
    audio = true;
  }

  let dir = temp_dir();
  let dest = unique_session_id();
  let filename = dest.to_owned()+".txt";
  let filename_audio = dest.to_owned()+"_audio.txt";
  let chunk_list = dir.join(filename);
  let chunk_list_audio = dir.join(filename_audio);
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
    if audio {
      let mut f = File::create(chunk_list_audio.to_owned()).expect("Unable to create file");
      let mut time = start;
      while time <= stop {
        let chunkf = to_path_aac(&storage, time);
        if chunkf.exists() {
          let s = fs::canonicalize(chunkf).unwrap().into_os_string().into_string().unwrap();
          let s = "file ".to_string()+&s+"\n";
          let _x = f.write(&s.as_bytes()).unwrap();
          b = true;
        }
        time += 2000;
      }
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
    o.push_string("-i");
    o.push_string(&chunk_list.to_owned().into_os_string().into_string().unwrap());
    if rot == 0 {
      o.push_string("-c:v");
      o.push_string("copy");
    }
    else {
      if settings.get_string("device") == "libcamera-apps".to_string(){
        o.push_string("-c:v");
        o.push_string("copy");
        o.push_string("-metadata:s:v:0");
        o.push_string(&("rotate=".to_string()+&(360-rot).to_string()));
      }
      else {
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
    }
    if !audio {
      o.push_string("-movflags");
      o.push_string("faststart");
    }
    
    let mut vidfile = generated.to_owned();
    if audio { vidfile = dir.join(dest.to_owned()+".mp4"); }
    o.push_string(&vidfile.to_owned().into_os_string().into_string().unwrap());
    //println!("{}", o.to_string());
    let _x = system_call(o);
    //println!("{}", _x.to_string());
    
    if audio {
      let mut o = DataArray::new();
      o.push_string("ffmpeg");
      o.push_string("-y");
      o.push_string("-f");
      o.push_string("concat");
      o.push_string("-safe");
      o.push_string("0");
      o.push_string("-i");
      o.push_string(&chunk_list_audio.to_owned().into_os_string().into_string().unwrap());
      o.push_string("-c:a");
      o.push_string("copy");
      let audiofile = dir.join(dest.to_owned()+".aac");
      o.push_string(&audiofile.to_owned().into_os_string().into_string().unwrap());
      //println!("{}", o.to_string());
      let _x = system_call(o);
      //println!("{}", _x.to_string());

      let mut o = DataArray::new();
      o.push_string("ffmpeg");
      o.push_string("-y");
      o.push_string("-i");
      o.push_string(&vidfile.to_owned().into_os_string().into_string().unwrap());
    
      if settings.has("itsoffset") {
        let itsoffset = settings.get_string("itsoffset");
        o.push_string("-itsoffset");
        o.push_string(&itsoffset);
      }

      o.push_string("-i");
      o.push_string(&audiofile.to_owned().into_os_string().into_string().unwrap());
      o.push_string("-c");
      o.push_string("copy");
      o.push_string("-movflags");
      o.push_string("faststart");
      o.push_string(&generated.to_owned().into_os_string().into_string().unwrap());
      //println!("{}", o.to_string());
      let _x = system_call(o);
      //println!("{}", _x.to_string());
      
      let _x = fs::remove_file(vidfile).unwrap();
      let _x = fs::remove_file(audiofile).unwrap();
    }
  }

  let _x = fs::remove_file(chunk_list).unwrap();
  if audio { let _x = fs::remove_file(chunk_list_audio).unwrap(); }
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

fn to_path_aac(storage:&str, time:i64) -> PathBuf {
  let now:DateTime<Utc> = Utc.timestamp_millis_opt(time).unwrap();
  let year = now.year();
  let month = now.month();
  let day = now.day();
  let hour = now.hour();
  let minute = now.minute();
  let second = now.second();
  let index = second / 2;
  let dir = Path::new(&storage).join("archive").join(year.to_string()).join(month.to_string()).join(day.to_string()).join(hour.to_string()).join(minute.to_string());
  let fname = index.to_string()+".aac";
  dir.join(fname)