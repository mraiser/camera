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