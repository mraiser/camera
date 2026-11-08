// C8a: a Mac or a Windows PC has neither rpicam nor v4l2; ffmpeg reaches
// their cameras (AVFoundation, DirectShow)
let os = std::env::consts::OS;
if os == "macos" || os == "windows" { return ffmpeg_cameras(os); }

let mut out = DataObject::new();

if let Some(vid) = camera_tool("vid") {
  let mut o = DataArray::new();
  o.push_string(&vid);
  o.push_string("--list-cameras");
  let x = system_call(o);
  if x.has("out") {
    let s = x.get_string("out");
    // rpicam-vid and libcamera-vid print the same shape:
    //   Available cameras
    //   -----------------
    //   0 : imx477 [4056x3040 10-bit RGGB] (/base/soc/i2c0mux/i2c@1/imx477@1a)
    //       Modes: 'SRGGB10_CSI2P' : 1332x990 [120.05 fps - (696, 528)/2664x1980 crop]
    //                                2028x1520 [40.01 fps - (0, 0)/4056x3040 crop]
    // Camera lines start unindented with "<index> :"; mode lines carry the
    // format in single quotes; each size is the first WxH token on its line.
    let mut started = false;
    let mut formats = DataObject::new();
    let mut sizes = DataArray::new();
    for line in s.lines() {
      if !started {
        if line.starts_with("-") { started = true; }
        continue;
      }
      let t = line.trim();
      if t.len() == 0 { continue; }
      let indented = line.starts_with(" ") || line.starts_with("\t");
      if !indented {
        let mut split = t.splitn(2, ':');
        let index = split.next().unwrap().trim();
        if index.len() > 0 && index.chars().all(|c| c.is_ascii_digit()) {
          if let Some(rest) = split.next() {
            formats = DataObject::new();
            let mut devices = DataObject::new();
            devices.put_object(&("libcamera:".to_string()+index), formats.clone());
            out.put_object(&("Raspberry Pi - ".to_string()+rest.trim()), devices);
            continue;
          }
        }
      }
      if let Some(q1) = t.find('\'') {
        if let Some(q2) = t[q1+1..].find('\'') {
          sizes = DataArray::new();
          formats.put_array(&t[q1+1..q1+1+q2], sizes.clone());
        }
      }
      for tok in t.split_whitespace() {
        let mut it = tok.split('x');
        if let (Some(a), Some(b)) = (it.next(), it.next()) {
          if it.next().is_none() {
            if let (Ok(w), Ok(h)) = (a.parse::<i64>(), b.parse::<i64>()) {
              let mut da = DataArray::new();
              da.push_int(w);
              da.push_int(h);
              sizes.push_array(da);
              break;
            }
          }
        }
      }
    }
  }
}

if out.objects().len() == 0 {
  let mut o = DataArray::new();
  o.push_string("v4l2-ctl");
  o.push_string("--list-devices");
  let x = system_call(o);
  // no usable v4l2-ctl output means no cameras - never hand the raw
  // system_call failure object to the UI as if it were the camera tree
  // (it renders as camera "err", resolution "Uxundefined")
  if !x.has("out") { return out; }

  let mut current = DataObject::new();
  let s = x.get_string("out");
  for line in s.lines() {
    if line.len() > 0 {
      if line.chars().nth(0).unwrap() as u16 == 9 {
        let line = line.trim();

        let mut o = DataArray::new();
        o.push_string("v4l2-ctl");
        o.push_string(&("--device=".to_string()+line));
        o.push_string("--list-formats-ext");
        let x = system_call(o);

        if x.has("out") {
          let mut o = DataObject::new();
          let mut b = false;

          let mut sizes = DataArray::new();
          let s = x.get_string("out");
          for line in s.lines() {
            let line = line.trim();
            if line.starts_with("[") {
              let vec = line.split("'").collect::<Vec<&str>>();
              let name = vec[1];

              sizes = DataArray::new();
              o.put_array(name, sizes.clone());
            }
            if line.starts_with("Size: ") {
              // Size: Stepwise 64x64 - 16384x16384 with step 2/2
              // Size: Discrete 1280x720
              let line = &line[6..];
              if line.starts_with("Stepwise ") {
                let line = &line[9..];
                let vec = line.split(" ").collect::<Vec<&str>>();
                let min = vec[0].split("x").collect::<Vec<&str>>();
                let max = vec[2].split("x").collect::<Vec<&str>>();
                let minx = min[0].parse::<i64>().unwrap();
                let miny = min[1].parse::<i64>().unwrap();
                let maxx = max[0].parse::<i64>().unwrap();
                let maxy = max[1].parse::<i64>().unwrap();

                let checks = [[1920,1080],[1280,720],[800,600],[640,480],[480,360],[320,240],[240,180]];
                for check in checks {
                  if check[0] >= minx && check[1] >= miny && check[0] <= maxx && check[1] <= maxy {
                    let mut da = DataArray::new();
                    da.push_int(check[0]);
                    da.push_int(check[1]);
                    sizes.push_array(da);
                    b = true;
                  }
                }
              }
              else if line.starts_with("Discrete ") {
                let line = &line[9..];
                let vec = line.split("x").collect::<Vec<&str>>();
                let x = vec[0].parse::<i64>().unwrap();
                let y = vec[1].parse::<i64>().unwrap();
                let mut da = DataArray::new();
                da.push_int(x);
                da.push_int(y);
                sizes.push_array(da);
                b = true;
              }
            }
          }
          if b { current.put_object(line, o.clone()); }
        }
      }
      else {
        current = DataObject::new();
        out.put_object(line, current.clone());
      }
    }
  }
}

out
}

// Raspberry Pi OS Bookworm+ ships rpicam-*; older releases libcamera-*.
// kind is "vid" or "still"; None when neither toolchain is installed.
pub fn camera_tool(kind:&str) -> Option<String> {
  for prefix in ["rpicam-", "libcamera-"] {
    let name = prefix.to_string()+kind;
    let mut o = DataArray::new();
    o.push_string("which");
    o.push_string(&name);
    let x = system_call(o);
    if x.has("out") && x.get_string("out").trim() != "" { return Some(name); }
  }
  None
}

// C8: cameras ffmpeg opens directly - AVFoundation on macOS, DirectShow on
// Windows. The device value names the input: "avfoundation:<index>" or
// "dshow:<name>". Neither hands over H.264, so the capture encodes
// (ffmpeg_encode_args) and available reports their one format as "H264":
// the rest of the app (live, play, the DVR segmenter) treats them as an
// H.264 camera.
pub fn is_avfoundation(device:&str) -> bool {
  device.starts_with("avfoundation:")
}

pub fn is_dshow(device:&str) -> bool {
  device.starts_with("dshow:")
}

pub fn is_ffmpeg_input(device:&str) -> bool {
  is_avfoundation(device) || is_dshow(device)
}

// The ffmpeg input arguments for an avfoundation/dshow device value, or
// None for anything else. Device values come off the URL (snapshot), so
// only the shapes available writes are accepted, each only on its own OS,
// and the value is always one argument (never through a shell).
pub fn ffmpeg_input_args(os:&str, device:&str, width:i64, height:i64, fps:i64) -> Option<Vec<String>> {
  let size = width.to_string()+"x"+&height.to_string();
  let fps = fps.max(1).to_string();
  if os == "macos" && is_avfoundation(device) {
    let index = &device["avfoundation:".len()..];
    if index.len() == 0 || !index.chars().all(|c| c.is_ascii_digit()) { return None; }
    // "<video>:none" - this camera, no audio
    let input = index.to_string()+":none";
    return Some(["-f", "avfoundation", "-framerate", &fps, "-video_size", &size, "-i", &input].iter().map(|s| s.to_string()).collect());
  }
  if os == "windows" && is_dshow(device) {
    let name = &device["dshow:".len()..];
    if name.trim().len() == 0 || name.contains('"') { return None; }
    let input = "video=".to_string()+name;
    return Some(["-f", "dshow", "-framerate", &fps, "-video_size", &size, "-i", &input].iter().map(|s| s.to_string()).collect());
  }
  None
}

// What the capture runs after ffmpeg_input_args: an H.264 elementary
// stream on stdout with a keyframe every 2 s (the DVR segmenter cuts 2 s
// segments with -c copy) and SPS/PPS repeated on every keyframe (live
// viewers join the fan-out mid-stream; rpicam's --inline does the same).
pub fn ffmpeg_encode_args(fps:i64) -> Vec<String> {
  let gop = (fps.max(1)*2).to_string();
  ["-c:v", "libx264", "-preset", "ultrafast", "-tune", "zerolatency", "-pix_fmt", "yuv420p",
   "-g", &gop, "-keyint_min", &gop, "-sc_threshold", "0", "-x264-params", "repeat-headers=1",
   "-an", "-f", "h264", "-"].iter().map(|s| s.to_string()).collect()
}

// ffmpeg prefixes each log line with a "[name @ 0x...] " tag; drop it.
fn untag(line:&str) -> &str {
  let t = line.trim_start();
  if t.starts_with('[') {
    if let Some(i) = t.find("] ") {
      if t[..i].contains(" @ ") { return &t[i+2..]; }
    }
  }
  t
}

// `ffmpeg -f avfoundation -list_devices true -i ""` (stderr):
//   [AVFoundation indev @ 0x7f9] AVFoundation video devices:
//   [AVFoundation indev @ 0x7f9] [0] FaceTime HD Camera
//   [AVFoundation indev @ 0x7f9] [1] Capture screen 0
//   [AVFoundation indev @ 0x7f9] AVFoundation audio devices:
//   [AVFoundation indev @ 0x7f9] [0] MacBook Pro Microphone
// Returns (index, name) for each camera; screens are not cameras.
pub fn parse_avfoundation_devices(s:&str) -> Vec<(String, String)> {
  let mut out = Vec::new();
  let mut video = false;
  for line in s.lines() {
    let t = untag(line).trim();
    if t.contains("AVFoundation video devices") { video = true; continue; }
    if t.contains("AVFoundation audio devices") { video = false; continue; }
    if !video || !t.starts_with('[') { continue; }
    if let Some(i) = t.find(']') {
      let index = &t[1..i];
      let name = t[i+1..].trim();
      if index.len() == 0 || !index.chars().all(|c| c.is_ascii_digit()) { continue; }
      if name.len() == 0 || name.starts_with("Capture screen") { continue; }
      out.push((index.to_string(), name.to_string()));
    }
  }
  out
}

// Asking avfoundation for a size the camera lacks (-video_size 1x1) makes
// ffmpeg list what it has, then fail:
//   [avfoundation @ 0x7f8] Selected video size (1x1) is not supported by the device.
//   [avfoundation @ 0x7f8] Supported modes:
//   [avfoundation @ 0x7f8]   1280x720@[1.000000 30.000000]fps
//   [avfoundation @ 0x7f8]   640x480@[1.000000 30.000000]fps
pub fn parse_avfoundation_modes(s:&str) -> Vec<(i64, i64)> {
  let mut out = Vec::new();
  let mut modes = false;
  for line in s.lines() {
    let t = untag(line).trim();
    if t.starts_with("Supported modes") { modes = true; continue; }
    if !modes { continue; }
    match t.split('@').next().and_then(parse_size) {
      Some(wh) => { if !out.contains(&wh) { out.push(wh); } }
      None => { if out.len() > 0 { break; } }
    }
  }
  out
}

// `ffmpeg -list_devices true -f dshow -i dummy` (stderr). ffmpeg 5 and
// later mark each device:
//   [dshow @ 000001] "Integrated Camera" (video)
//   [dshow @ 000001]   Alternative name "@device_pnp_\\?\usb#vid_..."
//   [dshow @ 000001] "Microphone (Realtek Audio)" (audio)
// older builds group them under headings instead:
//   [dshow @ 000001] DirectShow video devices (some may be both video and audio devices)
//   [dshow @ 000001]  "Integrated Camera"
//   [dshow @ 000001] DirectShow audio devices
pub fn parse_dshow_devices(s:&str) -> Vec<String> {
  let mut out:Vec<String> = Vec::new();
  let mut video = false;
  for line in s.lines() {
    let t = untag(line).trim();
    if t.starts_with("DirectShow video devices") { video = true; continue; }
    if t.starts_with("DirectShow audio devices") { video = false; continue; }
    if t.starts_with("Alternative name") || !t.starts_with('"') { continue; }
    if let Some(j) = t[1..].find('"') {
      let name = &t[1..1+j];
      let rest = t[2+j..].trim();
      let is_video = rest.contains("(video)") || (rest.len() == 0 && video);
      if is_video && name.len() > 0 && !out.iter().any(|n| n == name) { out.push(name.to_string()); }
    }
  }
  out
}

// `ffmpeg -list_options true -f dshow -i video=<name>` (stderr):
//   [dshow @ 000001]   vcodec=mjpeg  min s=1280x720 fps=30 max s=1280x720 fps=30
//   [dshow @ 000001]   pixel_format=yuyv422  min s=640x480 fps=5 max s=640x480 fps=30
pub fn parse_dshow_options(s:&str) -> Vec<(i64, i64)> {
  let mut out = Vec::new();
  for line in s.lines() {
    let t = untag(line);
    if !t.contains("min s=") { continue; }
    for tok in t.split_whitespace() {
      if let Some(v) = tok.strip_prefix("s=") {
        if let Some(wh) = parse_size(v) { if !out.contains(&wh) { out.push(wh); } }
      }
    }
  }
  out
}

fn parse_size(s:&str) -> Option<(i64, i64)> {
  let mut it = s.trim().split('x');
  match (it.next(), it.next(), it.next()) {
    (Some(w), Some(h), None) => match (w.parse::<i64>(), h.parse::<i64>()) {
      (Ok(w), Ok(h)) if w > 0 && h > 0 => Some((w, h)),
      _ => None
    },
    _ => None
  }
}

// An ffmpeg error that means macOS refused the camera to this process
// (System Settings > Privacy & Security > Camera, for the app that started
// Newbound - Terminal, iTerm, ...).
pub fn camera_access_denied(s:&str) -> bool {
  let l = s.to_lowercase();
  l.contains("not authorized") || l.contains("access denied") || l.contains("failed to create av capture input device")
}

// C8a: list cameras through ffmpeg on macOS (AVFoundation) and Windows
// (DirectShow), in the tree the UI reads: camera name -> device value ->
// format -> [[width, height], ...]. The one format is "H264" (see
// is_avfoundation). No ffmpeg, or a camera macOS won't open, is an error
// with the reason (the tile and dialog show it), never an empty tree.
pub fn ffmpeg_cameras(os:&str) -> DataObject {
  let mut out = DataObject::new();
  let mut o = DataArray::new();
  o.push_string("ffmpeg");
  o.push_string("-hide_banner");
  if os == "macos" {
    o.push_string("-f"); o.push_string("avfoundation");
    o.push_string("-list_devices"); o.push_string("true");
    o.push_string("-i"); o.push_string("");
  }
  else {
    o.push_string("-list_devices"); o.push_string("true");
    o.push_string("-f"); o.push_string("dshow");
    o.push_string("-i"); o.push_string("dummy");
  }
  let x = system_call(o);
  // system_call has no "out" only when the tool could not be started
  if !x.has("out") {
    if os == "macos" { panic!("ffmpeg isn't installed on this Mac, so Newbound can't reach its cameras. Install it (brew install ffmpeg), then restart Newbound."); }
    panic!("ffmpeg isn't installed on this PC, so Newbound can't reach its cameras. Install it (winget install ffmpeg) so it's on PATH, then restart Newbound.");
  }
  // ffmpeg prints device lists on stderr
  let listing = x.get_string("err");
  let cams:Vec<(String, String)> = if os == "macos" {
    parse_avfoundation_devices(&listing).into_iter().map(|(i, n)| ("avfoundation:".to_string()+&i, n)).collect()
  }
  else {
    parse_dshow_devices(&listing).into_iter().map(|n| ("dshow:".to_string()+&n, n)).collect()
  };

  for (device, name) in cams {
    let mut o = DataArray::new();
    o.push_string("ffmpeg");
    o.push_string("-hide_banner");
    if os == "macos" {
      // a size no camera has makes ffmpeg list the ones it does
      o.push_string("-f"); o.push_string("avfoundation");
      o.push_string("-video_size"); o.push_string("1x1");
      o.push_string("-i"); o.push_string(&(device["avfoundation:".len()..].to_string()+":none"));
    }
    else {
      o.push_string("-list_options"); o.push_string("true");
      o.push_string("-f"); o.push_string("dshow");
      o.push_string("-i"); o.push_string(&("video=".to_string()+&name));
    }
    let x = system_call(o);
    let err = if x.has("err") { x.get_string("err") } else { "".to_string() };
    let mut sizes = if os == "macos" { parse_avfoundation_modes(&err) } else { parse_dshow_options(&err) };
    if sizes.len() == 0 {
      if os == "macos" && camera_access_denied(&err) {
        panic!("macOS didn't let Newbound open {}. Allow the app Newbound runs in (Terminal, for example) under System Settings > Privacy & Security > Camera, then restart Newbound.", name);
      }
      // the modes couldn't be read: offer the common sizes rather than
      // hide a camera that is there
      sizes = vec![(1280, 720), (640, 480)];
    }
    let mut da = DataArray::new();
    for (w, h) in sizes {
      let mut s = DataArray::new();
      s.push_int(w);
      s.push_int(h);
      da.push_array(s);
    }
    let mut formats = DataObject::new();
    formats.put_array("H264", da);
    let mut devices = DataObject::new();
    devices.put_object(&device, formats);
    out.put_object(&name, devices);
  }
  out
}

// A libcamera-family device value: "libcamera:<index>" (written by
// available) or the legacy "libcamera-apps" (index 0) from older
// saved settings.
pub fn is_libcamera(device:&str) -> bool {
  device.starts_with("libcamera")
}

pub fn camera_index(device:&str) -> i64 {
  if let Some(i) = device.find(':') {
    if let Ok(n) = device[i+1..].parse::<i64>() { return n; }
  }
  0