use ndata::dataobject::*;
use ndata::dataarray::DataArray;
use flowlang::flowlang::system::system_call::system_call;

pub fn execute(_o: DataObject) -> DataObject {
let ax = available();
let mut o = DataObject::new();
o.put_object("a", ax);
o
}

pub fn available() -> DataObject {
let mut out = DataObject::new();
let mut ffmpeg = true;

let mut o = DataArray::new();
o.push_string("which");
o.push_string("libcamera-vid");
let x = system_call(o);
if x.get_string("out") != "".to_string() {
  let mut o = DataArray::new();
  o.push_string("libcamera-vid");
  o.push_string("--list-cameras");
  let x = system_call(o);
  if x.has("out") {
    let s = x.get_string("out");
    if s.starts_with("Available cameras"){
      ffmpeg = false;
      let mut o = DataObject::new();
      let mut b = false;
      let mut sizes = DataArray::new();
      for mut line in s.lines() {
        if b {
          let mut chars = line.chars();
          if chars.nth(0).unwrap() as u16 == 32 {
            if chars.nth(10).unwrap() as u16 == 39 {
              let mut split = line[12..].split("'");
              let name = &split.next().unwrap();
              sizes = DataArray::new();
              o.put_array(name, sizes.clone());
              line = &split.next().unwrap()[3..];
  //            println!("{:?}",name);
            }
            else { 
              line = line.trim(); 
            }
            line = line.split(" ").next().unwrap();
            let vec = line.split("x").collect::<Vec<&str>>();
            let x = vec[0].parse::<i64>().unwrap();
            let y = vec[1].parse::<i64>().unwrap();
            let mut da = DataArray::new();
            da.push_int(x);
            da.push_int(y);
            sizes.push_array(da);
          }
          else {
            let mut split = line.split(":");
            // FIXME - need to keep index to specify which camera to libcamera-apps
            let _i = split.next().unwrap().trim().parse::<usize>();
            let mut current = DataObject::new();
            o = DataObject::new();
            current.put_object("libcamera-apps", o.clone());
            out.put_object(&("Raspberry Pi - ".to_string()+&split.next().unwrap().trim()), current.clone());
          }
        }
        else {
          if line.starts_with("-"){
            b = true;
          }
        }
      }
    }
  }
}

if ffmpeg {
  let mut o = DataArray::new();
  o.push_string("v4l2-ctl");
  o.push_string("--list-devices");
  let x = system_call(o);
  if !x.has("out") { return x; }

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

