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
      if (result.status != 'ok') {
        // say why in the section, never a browser alert
        $(ME).find('.cam-fields').css('display', 'none');
        $(ME).find('.nocam-message').text("Couldn't list cameras: " + (result.msg || 'no reply.')).css('display', 'block');
      }
      else {
        me.available = result.data || {};
        var val = me.settings.camera;
        var newhtml = "";
        var count = 0;
        for (var i in me.available) {
          count++;
          var s = i == val ? ' selected' : '';
          newhtml += "<option"+s+">"+i+"</option>";
        }
        if (!count) {
          // C7: no camera - one sentence, no empty selects, no preview retries
          $(ME).find('.cam-fields').css('display', 'none');
          var el = $(ME).find('.nocam-message');
          el.css('display', 'block');
          // step 0c: the same fallback chain camera.live uses -
          // displayname, then name, then the peer id; never "undefined"
          if (!ME.DATA.peer) { el.text('No camera found on this machine.'); return; }
          el.text('No camera found.');
          json('../peer/peers', null, function(r){
            var list = (r && r.data && r.data.list) || (r && r.data) || [];
            var name = ME.DATA.peer;
            for (var i = 0; i < list.length; i++) {
              var p = list[i];
              if (p && (p.id == ME.DATA.peer || p.uuid == ME.DATA.peer)) { name = p.displayname || p.name || ME.DATA.peer; break; }
            }
            el.text('No camera found on ' + name + '.');
          });
          return;
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
    else if (me.settings.dvr) setTimeout(snapShot, 2000);
    else snapShot();
  });
  $(ME).find('#snapshot').on("error", function(){
    $(this).prop('src', me.prefix+'app/asset/camera/no_signal.jpg');
  });
};

function buildDevices() {
  var cam = $(ME).find('#select-camera').val();
  var dev = me.available[cam] || {};
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
  var format = (me.available[cam] || {})[dev] || {};
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
  var res = ((me.available[cam] || {})[dev] || {})[format] || [];
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
  // stop once the dialog is closed or replaced: a hidden preview must not
  // keep asking the camera for frames
  if (!ME.isConnected || !$(ME).is(':visible')) return;
  // C6: while recording, the capture owns the camera - preview its latest
  // keyframe instead of starting a second capture
  if (me.settings.dvr) {
    $(ME).find('#snapshot').prop("src", me.prefix+"camera/last_keyframe/y"+(q++)+".jpg");
    return;
  }
  var dev = $(ME).find('#select-device').val();
  var format = $(ME).find('#select-format').val();
  var res = $(ME).find('#select-resolution').val();
  var rot = $(ME).find('#select-rotation').val();
  // never emit malformed geometry - both parts must be positive numbers
  // ("Uxundefined" from a garbage tree passes an indexOf check)
  if (!dev || !res) { setTimeout(snapShot, 1000); return; }
  res = String(res).split("x");
  if (!(+res[0] > 0) || !(+res[1] > 0)) { setTimeout(snapShot, 1000); return; }
  $(ME).find('#snapshot').prop("src", me.prefix+"camera/snapshot/x"+(q++)+".jpg?device="+encodeURIComponent(dev)+"&format="+encodeURIComponent(format)+"&width="+res[0]+"&height="+res[1]+"&rot="+rot);
}