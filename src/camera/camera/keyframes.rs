use ndata::dataobject::*;
use crate::camera::camera::settings::settings;
use ndata::data::Data;

pub fn execute(o: DataObject) -> DataObject {
let a0 = o.get_string("nn_path");
let ax = keyframes(a0);
let mut o = DataObject::new();
o.put_string("a", &ax);
o
}

pub fn keyframes(nn_path:String) -> String {
let settings = settings(Data::DNull);
let storage = settings.get_string("storage");
let x = storage + "/keyframes/" + &nn_path[18..];
x
}

