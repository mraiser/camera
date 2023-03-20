let system = DataStore::globals().get_object("system");
let meta = system.get_object("apps").get_object("camera");
let settings = settings(Data::DNull);
let storage = settings.get_string("storage");

let recording = meta.has("last_keyframe") && time() - meta.get_int("last_keyframe") < 4000;

let mut o = settings.deep_copy();
o.put_object("app", meta.get_object("app"));
o.put_boolean("recording", recording);

let keyframes = Path::new(&storage).join("keyframes");
if keyframes.exists(){
  let mut first_year:u16 = 9999;
  let mut last_year = 0;
  for f in fs::read_dir(&keyframes).unwrap(){
    let f = f.unwrap().path();
    let name = f.file_name().unwrap().to_str().unwrap();
    let num = name.parse::<u16>();
    if num.is_ok() {
      let year = num.unwrap();
      if year < first_year { first_year = year; }
      if year > last_year { last_year = year; }
    }
  }

  if first_year != 9999 {
    let mut first_month = 99;
    let dir = keyframes.join(first_year.to_string());
    for f in fs::read_dir(&dir).unwrap(){
      let f = f.unwrap().path();
      let name = f.file_name().unwrap().to_str().unwrap();
      let num = name.parse::<u8>();
      if num.is_ok() {
        let month = num.unwrap();
        if month < first_month { first_month = month; }
      }
    }

    if first_month != 99 {
      let mut first_day = 99;
      let dir = dir.join(first_month.to_string());
      for f in fs::read_dir(&dir).unwrap(){
        let f = f.unwrap().path();
        let name = f.file_name().unwrap().to_str().unwrap();
        let num = name.parse::<u8>();
        if num.is_ok() {
          let day = num.unwrap();
          if day < first_day { first_day = day; }
        }
      }

      if first_day != 99 {
        let mut first_hour = 99;
        let dir = dir.join(first_day.to_string());
        for f in fs::read_dir(&dir).unwrap(){
          let f = f.unwrap().path();
          let name = f.file_name().unwrap().to_str().unwrap();
          let num = name.parse::<u8>();
          if num.is_ok() {
            let hour = num.unwrap();
            if hour < first_hour { first_hour = hour; }
          }
        }

        if first_hour != 99 {
          let mut first_minute = 99;
          let dir = dir.join(first_hour.to_string());
          for f in fs::read_dir(&dir).unwrap(){
            let f = f.unwrap().path();
            let name = f.file_name().unwrap().to_str().unwrap();
            let num = name.parse::<u8>();
            if num.is_ok() {
              let minute = num.unwrap();
              if minute < first_minute { first_minute = minute; }
            }
          }

          if first_minute != 99 {
            let mut first_second = 99;
            let dir = dir.join(first_minute.to_string());
            for f in fs::read_dir(&dir).unwrap(){
              let f = f.unwrap().path();
              let name = f.file_name().unwrap().to_str().unwrap();
              let name = &name[..name.len()-4];
              let num = name.parse::<u8>();
              if num.is_ok() {
                let second = num.unwrap();
                if second < first_second { first_second = second; }
              }
            }
            if first_second != 99 {
              let timestamp = Utc.with_ymd_and_hms(first_year as i32, first_month as u32, first_day as u32, first_hour as u32, first_minute as u32, first_second as u32).unwrap();
              let timestamp = timestamp.timestamp_millis();
              o.put_int("first", timestamp);
            }
          }
        }
      }
    }
  }

  if last_year != 0 {
    let mut last_month = 0;
    let dir = keyframes.join(last_year.to_string());
    for f in fs::read_dir(&dir).unwrap(){
      let f = f.unwrap().path();
      let name = f.file_name().unwrap().to_str().unwrap();
      let num = name.parse::<u8>();
      if num.is_ok() {
        let month = num.unwrap();
        if month > last_month { last_month = month; }
      }
    }

    if last_month != 0 {
      let mut last_day = 0;
      let dir = dir.join(last_month.to_string());
      for f in fs::read_dir(&dir).unwrap(){
        let f = f.unwrap().path();
        let name = f.file_name().unwrap().to_str().unwrap();
        let num = name.parse::<u8>();
        if num.is_ok() {
          let day = num.unwrap();
          if day > last_day { last_day = day; }
        }
      }

      if last_day != 0 {
        let mut last_hour = 0;
        let dir = dir.join(last_day.to_string());
        for f in fs::read_dir(&dir).unwrap(){
          let f = f.unwrap().path();
          let name = f.file_name().unwrap().to_str().unwrap();
          let num = name.parse::<u8>();
          if num.is_ok() {
            let hour = num.unwrap();
            if hour > last_hour { last_hour = hour; }
          }
        }

        if last_hour != 0 {
          let mut last_minute = 0;
          let dir = dir.join(last_hour.to_string());
          for f in fs::read_dir(&dir).unwrap(){
            let f = f.unwrap().path();
            let name = f.file_name().unwrap().to_str().unwrap();
            let num = name.parse::<u8>();
            if num.is_ok() {
              let minute = num.unwrap();
              if minute > last_minute { last_minute = minute; }
            }
          }

          if last_minute != 0 {
            let mut last_second = 0;
            let dir = dir.join(last_minute.to_string());
            for f in fs::read_dir(&dir).unwrap(){
              let f = f.unwrap().path();
              let name = f.file_name().unwrap().to_str().unwrap();
              let name = &name[..name.len()-4];
              let num = name.parse::<u8>();
              if num.is_ok() {
                let second = num.unwrap();
                if second > last_second { last_second = second; }
              }
            }
            if last_second != 0 {
              let timestamp = Utc.with_ymd_and_hms(last_year as i32, last_month as u32, last_day as u32, last_hour as u32, last_minute as u32, last_second as u32).unwrap();
              let timestamp = timestamp.timestamp_millis();
              o.put_int("last", timestamp);
            }
          }
        }
      }
    }
  }
}

o