let settings = settings(Data::DNull);
let storage = settings.get_string("storage");
let mp4path = storage.to_owned() + &mp4;
let jpgpath = storage.to_owned() + &jpg;

//let inline_snaps = settings.has("inline_snaps") && Data::as_string(settings.get_property("inline_snaps")) == "true".to_string();
//if !inline_snaps {
  let mut meta = DataStore::globals().get_object("system").get_object("apps").get_object("camera");
  if !meta.has("SNAPPING"){
    meta.put_boolean("SNAPPING", true);

    let mut o = DataArray::new();

    let device = settings.get_string("device");
    let binding = settings.get_string("resolution");
    let res = binding.split("x").collect::<Vec<&str>>();
    let width = res[0].parse::<i64>().unwrap();
    let height = res[1].parse::<i64>().unwrap();
    let rot = settings.get_int("rotation");

    let t = match rot {
      90 => "transpose=1",
      180 => "transpose=2,transpose=2",
      270 => "transpose=2",
      _ => "transpose=none",
    };

    let mut t = t.to_string();

    if !is_libcamera(&device) {
      if rot == 90 || rot == 270 {
        let q = (width*width)/height;
        t = t + ",scale=" + &width.to_string()+":"+&q.to_string()+",crop="+&width.to_string()+":"+&height.to_string();
      }
    }

    o.push_string("ffmpeg");
    // a keyframe that already exists (a restarted recording reuses segment
    // numbers) made ffmpeg ask "Overwrite? [y/N]" on the server's terminal
    // and wait there for ever: overwrite, and never read stdin
    o.push_string("-y");
    o.push_string("-nostdin");
    o.push_string("-i");
    o.push_string(&mp4path);
    o.push_string("-update");
    o.push_string("1");
    o.push_string("-vframes:v");
    o.push_string("1");
    if rot != 0 {
      o.push_string("-filter:v");
      o.push_string(&t);
    }

    o.push_string(&jpgpath);

    //for el in o.objects() { print!("{} ", el.string()); } println!("");

    let _x = system_call(o);

    meta.remove_property("SNAPPING");
    meta.put_string("last_jpg", &jpgpath);
  }
//}

if settings.has("motion") && settings.get_boolean("motion"){
  let img = image::open(&jpgpath);
  if img.is_ok() {
    let system = DataStore::globals().get_object("system");
    let mut meta = system.get_object("apps").get_object("camera");
    
    let sense = settings.get_int("motion_sensitivity");
    let noise = settings.get_int("motion_noise_cancel");
    let factor:f32 = noise as f32 / 2.0;
    let maxx:u32 = ((((sense as f32/55.0)-0.2) * -1.0) as f32 * 256.0) as u32;
    
    let img = img.unwrap();
    let img = img.resize_exact(MOTION_SIZE, MOTION_SIZE, FilterType::Nearest);
    
    let mut score:u32 = 0;
    let mut adj:u32 = 0;
    let mut noise:u32 = 0;

    // delta/delta2 were per-frame scratch in the old static arrays; only
    // the previous frame (last) and the noise model (map) persist.
    let mut guard = motion_state().lock().unwrap_or_else(|e| e.into_inner());
    let state = &mut *guard;

    for (x, y, pixel) in img.pixels() {
      let idx = (x as usize) * (MOTION_SIZE as usize) + (y as usize);
      let rgb = pixel.to_rgb();
      let last = &mut state.last[idx];
      let map = &mut state.map[idx];

      let mut i = 0;
      while i < 3 {
        let delta = (last[i] as i16 - rgb[i] as i16).abs() as u8;
        let delta2 = cmp::max(delta as i16 - (factor * map[i].get_average() as f32) as i16, 0) as u8;
        map[i].add_sample(delta as u32);

        score += delta as u32;
        adj += delta2 as u32;
        noise += map[i].get_average();

        last[i] = rgb[i];
        i += 1;
      }
    }

    score /= MOTION_COUNT;
    adj /= MOTION_COUNT;
    noise /= MOTION_COUNT;

    if adj > maxx {
      let mut event_frame = DataObject::new();
      event_frame.put_int("time", timestamp);
      event_frame.put_int("score", score as i64);
      event_frame.put_int("adjusted", adj as i64);
      event_frame.put_int("noise", noise as i64);
      event_frame.put_string("mp4", &mp4);
      event_frame.put_string("jpg", &jpg);

      let mut event;
      if meta.has("current_event") { event = meta.get_array("current_event"); }
      else {
        event = DataArray::new();
        meta.put_array("current_event", event.clone());
        fire_event("camera", "motion_begin", event_frame.clone());
      }

      event.push_object(event_frame);
    }
    else {
      if meta.has("current_event") {
        let event = meta.get_array("current_event");
        meta.remove_property("current_event");
        let mut event_frame = event.get_object(0).deep_copy();
        event_frame.put_array("frames", event);
        fire_event("camera", "motion_end", event_frame.clone());

        let f = to_path(&storage, timestamp, "events", "json");
        let mut f = File::create(f).unwrap();
        let _x = f.write_all(event_frame.to_string().as_bytes());
      }
    }
  }
}




"OK".to_string()
}

const MOTION_SIZE:u32 = 64;
const MOTION_COUNT:u32 = MOTION_SIZE * MOTION_SIZE;
struct MotionState {
  last: Vec<[u8; 3]>,
  map: Vec<[SingleSumSMA<u32, u32, 10>; 3]>,
}

// Serialized, safe motion state. The old form - static mut Vecs mutated
// through unsafe references - was a data race between overlapping
// keyframe events and a hard error under Rust edition 2024.
fn motion_state() -> &'static Mutex<MotionState> {
  static MOTION: OnceLock<Mutex<MotionState>> = OnceLock::new();
  MOTION.get_or_init(|| Mutex::new(MotionState {
    last: vec![[0u8; 3]; MOTION_COUNT as usize],
    map: (0..MOTION_COUNT).map(|_| [SingleSumSMA::new(), SingleSumSMA::new(), SingleSumSMA::new()]).collect(),
  }))
}

pub fn to_path(storage:&str, time:i64, folder:&str, suffix:&str) -> PathBuf {
  let now:DateTime<Utc> = Utc.timestamp_millis_opt(time).unwrap();
  let year = now.year();
  let month = now.month();
  let day = now.day();
  let hour = now.hour();
  let minute = now.minute();
  let second = now.second();
  let index = second / 2;
  let dir = Path::new(&storage).join(folder).join(year.to_string()).join(month.to_string()).join(day.to_string()).join(hour.to_string()).join(minute.to_string());
  let _x = create_dir_all(&dir);
  let fname = index.to_string()+"."+suffix;
  dir.join(fname)