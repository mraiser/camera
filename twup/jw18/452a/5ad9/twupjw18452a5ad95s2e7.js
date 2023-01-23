var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  me.status = ME.DATA
  if (!ME.DATA.peer) me.prefix = "../";
  else me.prefix = "../peer/remote/"+ME.DATA.peer+"/";
  if (ME.DATA.controls == false) $(ME).find('.live-img-ctrls').css('display', 'none');
  if (me.status.device) next();
  else {
    json(me.prefix+'camera/status', null, function(result){
      var peer = ME.DATA.peer;
      if (result.data) {
        me.status = ME.DATA = result.data;
        me.status.peer = peer;
        next();
      }
      else setTimeout(me.ready, 1000);
    });
  }
};

var q = 0;
function next(){
  if (me.pause) { setTimeout(next, 1000); }
  else if (me.status.dvr) {
    $(ME).find('.live-img').prop("src", me.prefix+"camera/last_keyframe/y"+(q++)+".jpg");
  }
  else {
    var dev = me.status.device;
    var format = me.status.format;
    var res = me.status.resolution;
    var rot = me.status.rotation;
    if (res) res = res.split("x");
    else res = me.status.resolution = ['1920','1080'];
    $(ME).find('.live-img').prop("src", me.prefix+"camera/snapshot/x"+(q++)+".jpg?device="+encodeURIComponent(dev)+"&format="+encodeURIComponent(format)+"&width="+res[0]+"&height="+res[1]+"&rot="+rot);
  }
}

$(ME).find('.live-img').on("load", function(){
//  var r = ($(ME).width() - $(this).width()) / 2; 
//  $(ME).find('.live-img-ctrls').css('right', (r+20)+'px');
  setTimeout(next, 2000);
});

$(ME).find('.live-img').on("error", function(){
  $(this).prop('src', '../app/asset/camera/no_signal.jpg');
});

$(ME).find('.dvr-settings-button').click(function(e){
  me.pause = true;
  e.selector = ".settings-popup";
  e.closeselector = ".close-camera-settings";
  e.modal = true;
  e.close = function() { 
    json(me.prefix+'camera/status', null, function(result){
      me.status = ME.DATA = result.data;
      me.pause = false; 
      
      if (result.data.dvr) {
        $('.img-wrap').css('height', 'calc(100% - 45px)');
        $('.dvr-navbar').css('display', 'block');
      }
      else {
        $('.dvr-navbar').css('display', 'none');
        $('.img-wrap').css('height', '100%)');
      }
    });
  };
  
  document.body.api.ui.popup(e, function(){
    me.status.peer = ME.DATA.peer;
    installControl($(ME).find(".settings-popup-inner")[0], "camera", "dvr_settings", function(){}, me.status);
  });
});

$(ME).find('.fullscreen-button').click(function(){
  var elem = $(ME).find('.live-wrap')[0];
  if (elem.requestFullscreen) {
    elem.requestFullscreen();
  } else if (elem.webkitRequestFullscreen) { /* Safari */
    elem.webkitRequestFullscreen();
  } else if (elem.msRequestFullscreen) { /* IE11 */
    elem.msRequestFullscreen();
  }
  $(this).css('display', 'none');
  $(ME).find('.fullscreen-exit-button').css('display', 'inline-block');
  $(ME).find('.dvr-settings-button').css('display', 'none');
});


$(ME).find('.fullscreen-exit-button').click(function(){
  if (document.exitFullscreen) {
    document.exitFullscreen();
  } else if (document.webkitExitFullscreen) { /* Safari */
    document.webkitExitFullscreen();
  } else if (document.msExitFullscreen) { /* IE11 */
    document.msExitFullscreen();
  }
  $(this).css('display', 'none');
  $(ME).find('.dvr-settings-button').css('display', 'inline-block');
  $(ME).find('.fullscreen-button').css('display', 'inline-block');
});