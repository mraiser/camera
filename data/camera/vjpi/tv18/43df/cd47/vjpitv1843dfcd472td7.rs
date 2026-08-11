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