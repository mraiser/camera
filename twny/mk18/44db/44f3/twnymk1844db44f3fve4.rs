let mut o = DataArray::new();

let now:DateTime<Utc> = Utc.timestamp_millis_opt(timestamp).unwrap();
let year = now.year();
let month = now.month();
let day = now.day();
let hour = now.hour();
let minute = now.minute();
let second = now.second();
let index = second / 2;

let settings = settings(Data::DNull);
let device = settings.get_string("device");
let binding = settings.get_string("resolution");
let res = binding.split("x").collect::<Vec<&str>>();
let width = res[0].parse::<i64>().unwrap();
let height = res[1].parse::<i64>().unwrap();
let rot = settings.get_int("rotation");
let sense = settings.get_int("motion_sensitivity");
let noise = settings.get_int("motion_noise_cancel");
let storage = settings.get_string("storage");

let factor:f32 = noise as f32 / 2.0;
let maxx:u32 = ((((sense as f32/55.0)-0.2) * -1.0) as f32 * 256.0) as u32;

let dir = Path::new(&storage).join("keyframes").join(year.to_string()).join(month.to_string()).join(day.to_string()).join(hour.to_string()).join(minute.to_string());
let _x = fs::create_dir_all(&dir);
let f = dir.join(index.to_string()+".jpg");

let t = match rot {
  90 => "transpose=1",
  180 => "transpose=2,transpose=2",
  270 => "transpose=2",
  _ => "transpose=none",
};

let mut t = t.to_string();

if device == "libcamera-apps".to_string(){
}
else {
  if rot == 90 || rot == 270 {
    let q = (width*width)/height;
    t = t + ",scale=" + &width.to_string()+":"+&q.to_string()+",crop="+&width.to_string()+":"+&height.to_string();
  }
}

o.push_string("ffmpeg");
o.push_string("-i");
o.push_string(&path);
o.push_string("-update");
o.push_string("1");
o.push_string("-vframes:v");
o.push_string("1");
if rot != 0 {
  o.push_string("-filter:v");
  o.push_string(&t);
}

let path2 = f.into_os_string().into_string().unwrap();
o.push_string(&path2);

let _x = system_call(o);

let system = DataStore::globals().get_object("system");
let mut meta = system.get_object("apps").get_object("camera");
meta.put_string("last_mp4", &path);
meta.put_string("last_jpg", &path2);
meta.put_int("last_keyframe", timestamp);

let plen = storage.len();
let rpath = &path[plen..];
let rpath2 = &path2[plen..];

let mut x = DataObject::new();
x.put_string("mp4", rpath);
x.put_string("jpg", rpath2);
x.put_int("timestamp", timestamp);
fire_event("camera", "keyframe", x);

if settings.has("motion") && settings.get_boolean("motion"){
  START.call_once(|| { init(); });
  
  let img = image::open(&path2);
  if img.is_ok() {
    let img = img.unwrap();
    let img = img.resize_exact(MOTION_SIZE, MOTION_SIZE, FilterType::Nearest);

    let mut score:u32 = 0;
    let mut adj:u32 = 0;
    let mut noise:u32 = 0;

    // FIXME - Comment out if map file generation is not desired
    //let mut output = ImageBuffer::new(MOTION_SIZE, MOTION_SIZE);
    //for (x, y, mut pixel) in img.pixels() {

    for (x, y, pixel) in img.pixels() {
      let x = x as usize;
      let y = y as usize;
      let rgb = pixel.to_rgb();

      let last;
      let map;
      let delta;
      let delta2;

      unsafe { 
        last = &mut LAST[x][y]; 
        map = &mut MAP[x][y]; 
        delta = &mut DELTA[x][y]; 
        delta2 = &mut DELTA2[x][y]; 
      }

      let mut i = 0;
      while i < 3 {
        delta[i] = (last[i] as i16 - rgb[i] as i16).abs() as u8;
        delta2[i] = cmp::max(delta[i] as i16 - (factor * map[i].get_average() as f32) as i16, 0) as u8;
        map[i].add_sample(delta[i] as u32);

        score += delta[i] as u32;
        adj += delta2[i] as u32;
        noise += map[i].get_average();

        last[i] = rgb[i];

        // FIXME - Comment out if map file generation is not desired
        //pixel[i] = delta2[i];

        i += 1;
      }
      // FIXME - Comment out if map file generation is not desired
      //output.put_pixel(x as u32, y as u32, pixel);
    }

    score /= MOTION_COUNT;
    adj /= MOTION_COUNT;
    noise /= MOTION_COUNT;

    if adj > maxx {
  //    println!("MOTION DETECTED max {} factor {} score {} adj {} noise {}", maxx, factor, score, adj, noise);

      let mut event_frame = DataObject::new();
      event_frame.put_int("time", timestamp);
      event_frame.put_int("score", score as i64);
      event_frame.put_int("adjusted", adj as i64);
      event_frame.put_int("noise", noise as i64);
      event_frame.put_string("mp4", rpath);
      event_frame.put_string("jpg", rpath2);

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

        let dir = Path::new(&storage).join("events").join(year.to_string()).join(month.to_string()).join(day.to_string()).join(hour.to_string()).join(minute.to_string());
        let _x = fs::create_dir_all(&dir);
        let f = dir.join(index.to_string()+".json");
        let mut f = File::create(f).unwrap();
        let _x = f.write_all(event_frame.to_string().as_bytes());
      }
    }

    // FIXME - Comment out if map file generation is not desired
    //let f = dir.join(index.to_string()+"_map.jpg");
    //output.save(f);
  }
}

"OK".to_string()
}

const MOTION_SIZE:u32 = 64;
const MOTION_COUNT:u32 = MOTION_SIZE * MOTION_SIZE;
static START: Once = Once::new();
static mut LAST:Vec<Vec<Vec<u8>>> = Vec::new();
static mut MAP:Vec<Vec<Vec<SingleSumSMA::<u32, u32, 10>>>> = Vec::new();
static mut DELTA:Vec<Vec<Vec<u8>>> = Vec::new();
static mut DELTA2:Vec<Vec<Vec<u8>>> = Vec::new();

fn init(){
  let n = MOTION_SIZE;
  let mut x = 0;
  while x<n {
    let mut va = Vec::new();
    let mut vb = Vec::new();
    let mut vc = Vec::new();
    let mut vd = Vec::new();
    let mut y = 0;
    while y<n{
      let mut v2a = Vec::new();
      let mut v2b = Vec::new();
      let mut v2c = Vec::new();
      let mut v2d = Vec::new();
      let mut i = 0;
      while i<3{
        v2a.push(0);
        v2b.push(SingleSumSMA::new());
        v2c.push(0);
        v2d.push(0);
        i += 1;
      }
      va.push(v2a);
      vb.push(v2b);
      vc.push(v2c);
      vd.push(v2d);
      y += 1;
    }
    unsafe { 
      LAST.push(va); 
      MAP.push(vb); 
      DELTA.push(vc); 
      DELTA2.push(vd); 
    }
    x += 1;
  }