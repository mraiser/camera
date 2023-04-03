use ndata::dataobject::*;
use chrono::DateTime;
use chrono::Utc;
use chrono::TimeZone;
use chrono::Datelike;
use chrono::Timelike;
use std::path::Path;
use crate::camera::camera::settings::settings;
use ndata::data::Data;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_int("timestamp");
let ax = keyframe(a0);
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn keyframe(timestamp:i64) -> String {
let settings = settings(Data::DNull);
let storage = settings.get_string("storage");

let now:DateTime<Utc> = Utc.timestamp_millis_opt(timestamp).unwrap();
let year = now.year();
let month = now.month();
let day = now.day();
let hour = now.hour();
let minute = now.minute();
let second = now.second();
let index = second / 2;

Path::new(&storage)
	.join("keyframes")
    .join(year.to_string())
    .join(month.to_string())
    .join(day.to_string())
    .join(hour.to_string())
    .join(minute.to_string())
    .join(index.to_string()+".jpg")
    .into_os_string()
    .into_string()
    .unwrap()
}

