use ndata::dataobject::*;
use flowlang::datastore::DataStore;
use ndata::databytes::DataBytes;
use flowlang::flowlang::system::unique_session_id::unique_session_id;
use ndata::data::Data;
use crate::camera::camera::settings::settings;
use std::thread;
use std::process::Command;
use std::process::Stdio;
use core::time::Duration;
use std::io::Read;
use std::io::Write;
use flowlang::flowlang::system::time::time;

pub fn execute(_o: DataObject) -> DataObject {
let ax = live();
let mut o = DataObject::new();
o.put_bytes("a", ax);
o
}

pub fn live() -> DataBytes {
let system = DataStore::globals().get_object("system");
let meta = system.get_object("apps").get_object("camera");
let mut streams = meta.get_object("streams");

let inputstream = DataBytes::new();
let id = unique_session_id();
streams.put_bytes(&id, inputstream.clone());

let outputstream = DataBytes::new();
let o2 = outputstream.clone();

thread::spawn(move || {
  let settings = settings(Data::DNull);
  let fps = settings.get_int("framerate");
  let args1 = ["-framerate", &fps.to_string(), "-i", "-", "-movflags", "frag_keyframe+empty_moov", "-f", "mp4", "-"];
  let args1 = args1.to_vec();
  print!("{}", "ffmpeg"); for el in &args1 { print!(" {}", el); } println!("");
  let cmd1 = Command::new("ffmpeg")
    .args(args1)
    .stdin(Stdio::piped())
    .stderr(Stdio::null())
    .stdout(Stdio::piped())
    .spawn();
  if cmd1.is_ok() {
    let mut cmd1 = cmd1.unwrap();
    thread::spawn(move || {
      println!("HTTP write b begin");
      let reader = cmd1.stdout.as_mut().unwrap();
      let mut buf = [0; 409600];
      let timeout = time() + 10000;
      let beat = Duration::from_millis(10);
      loop {
        let mut b = false;
        {
          let x = reader.read(&mut buf);
          if x.is_err() {  break; }
          let x = x.unwrap();
          if x == 0 {
            if timeout < time() {
              break; 
            }      
            else {
              b = true;
            }
          }
          else {
            //println!("write b {}", x);
            o2.write(&buf[0..x]);
          }
        }
        if b { thread::sleep(beat); }
      }
      println!("HTTP write b end");
    });
    
    println!("HTTP write a begin");
    let writer1 = cmd1.stdin.as_mut().unwrap();
    let beat = Duration::from_millis(10);
    while inputstream.is_read_open() {
      let vec = inputstream.read(409600);
      if vec.len() > 0 {
        let x1 = writer1.write(&vec);
        if x1.is_err() {  break; }
        //println!("write a {}", vec.len());
      }
      else {
        thread::sleep(beat);
      }
    }
    println!("HTTP write a end");
  }
});


outputstream
}

