var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  me.settings = ME.DATA
  
  if (!ME.DATA.peer) me.prefix = "../";
  else me.prefix = "../peer/remote/"+ME.DATA.peer+"/";
  
//  json('../camera/settings', null, function(result) {
//    if (result.data) me.settings = result.data;
//    else me.settings = {};
    json(me.prefix+'camera/available', null, function(result){
      if (result.status != 'ok') { alert(result.msg); }
      else {
        me.available = result.data;
        var val = me.settings.camera;
        var newhtml = "";
        for (var i in result.data) {
          var s = i == val ? ' selected' : '';
          newhtml += "<option"+s+">"+i+"</option>";
        }
        $(ME).find('#select-camera').html(newhtml).on('change', buildDevices);
        buildDevices();
        snapShot();
        $(ME).find('#select-rotation').val(me.settings.rotation);
        $(ME).find('#select-fps').val(me.settings.framerate);
      }
    });
//  });
  $(ME).find('#snapshot').on("load", function(){
    if ($(this).prop('src').indexOf('no_signal.jpg') != -1) setTimeout(snapShot, 1000);
    else snapShot();
  });
  $(ME).find('#snapshot').on("error", function(){
    $(this).prop('src', me.prefix+'app/asset/camera/no_signal.jpg');
  });
};

function buildDevices() {
  var cam = $(ME).find('#select-camera').val();
  var dev = me.available[cam];
  var val = me.settings.device;
  var newhtml = "";
  for (var i in dev) {
    var s = i == val ? ' selected' : '';
    newhtml += "<option"+s+">"+i+"</option>";
  }
  $(ME).find('#select-device').html(newhtml).on('change', buildFormat);
  buildFormat();
}

function buildFormat() {
  var cam = $(ME).find('#select-camera').val();
  var dev = $(ME).find('#select-device').val();
  var format = me.available[cam][dev];
  var val = me.settings.format;
  var newhtml = "";
  for (var i in format) {
    var s = i == val ? ' selected' : '';
    newhtml += "<option"+s+">"+i+"</option>";
  }
  $(ME).find('#select-format').html(newhtml).on('change', buildResolution);
  buildResolution();
}

function buildResolution() {
  var cam = $(ME).find('#select-camera').val();
  var dev = $(ME).find('#select-device').val();
  var format = $(ME).find('#select-format').val();
  var res = me.available[cam][dev][format];
  var z = me.settings.resolution;
  var newhtml = "";
  for (var i in res) {
    var val = res[i][0]+'x'+res[i][1];
    var s = val == z ? ' selected' : '';
    newhtml += "<option"+s+">"+val+"</option>";
  }
  $(ME).find('#select-resolution').html(newhtml).on('change');
}

var q = 0;
function snapShot() {
  var dev = $(ME).find('#select-device').val();
  var format = $(ME).find('#select-format').val();
  var res = $(ME).find('#select-resolution').val();
  var rot = $(ME).find('#select-rotation').val();
  res = res.split("x");
  $(ME).find('#snapshot').prop("src", me.prefix+"camera/snapshot/x"+(q++)+".jpg?device="+encodeURIComponent(dev)+"&format="+encodeURIComponent(format)+"&width="+res[0]+"&height="+res[1]+"&rot="+rot);
}