var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (!ME.DATA.peer) me.prefix = "../";
  else me.prefix = "../peer/remote/"+ME.DATA.peer+"/";
  
  document.body.api.ui.initSliders(ME);
  $('.hideme').css('display', 'none');
  $(ME).find('#camera-settings-go-here').html('');
  json(me.prefix+'camera/settings', null, function(result) {
    if (result.data) {
      me.settings = result.data;
      $(ME).find('.rvs-sense').val(me.settings.motion_sensitivity).trigger("input");
      $(ME).find('.rvs-noise').val(me.settings.motion_noise_cancel).trigger("input");
      $(ME).find('#dvr-switch').prop('checked', me.settings.dvr);
      $(ME).find('#motion-switch').prop('checked', me.settings.motion);
      $(ME).find('.storagedir').val(me.settings.storage);
      $(ME).find('#select-fps').val(me.settings.framerate);
    }
    else me.settings = {};
    
    if (me.settings.dvr) {
      $(ME).find('.camera-settings-button').css('display', 'block');
      $(ME).find('.dvr-settings-go-here').css('display', 'block');
      $(ME).find('.hide-on-cam').css('display', 'block');
      if (me.settings.motion) {
        $(ME).find('.motion-settings').css('display', 'block');
      }
      $(ME).find('#keyframe').css('display', 'block');
      keyFrame();
    }
    else {
      $(ME).find('.dvr-settings-message').css('display', 'block');
      me.settings.peer = ME.DATA.peer;
      installControl('#camera-settings-go-here', 'camera', 'camera_settings', function(api){}, me.settings);
    }
  });
};

var q = 0;
function keyFrame() {
  if (me.settings.dvr) {
    $(ME).find('#keyframe').prop("src", me.prefix+"camera/last_keyframe/y"+(q++)+".jpg");
  }
}

$(ME).find('.save-camera-settings').click(function(){
  ME.DATA.camera = $(ME).find('#select-camera').val();
  ME.DATA.device = $(ME).find('#select-device').val();
  ME.DATA.format = $(ME).find('#select-format').val();
  ME.DATA.resolution = $(ME).find('#select-resolution').val();
  ME.DATA.rotation = $(ME).find('#select-rotation').val();
  ME.DATA.framerate = $(ME).find('#select-fps').val();
  ME.DATA.dvr = $(ME).find('#dvr-switch').prop('checked');
  ME.DATA.motion_sensitivity = $(ME).find('.rvs-sense').val();
  ME.DATA.motion_noise_cancel = $(ME).find('.rvs-noise').val();
  ME.DATA.motion = $(ME).find('#motion-switch').prop('checked');
  ME.DATA.recording = ME.DATA.dvr;
  json(me.prefix+'camera/settings', "settings="+encodeURIComponent(JSON.stringify(ME.DATA)), function(result) {
    $('.close-camera-settings').click();
  });
});

$(ME).find('#motion-switch').on('change', function(){
  var dis = $(this).prop('checked') ? 'block' : 'none';
  $(ME).find('.motion-settings').css('display', dis);
});

$(ME).find('#keyframe').on("load", function(){
  if ($(this).prop('src').indexOf('no_signal.jpg') != -1) setTimeout(keyFrame, 1000);
  else setTimeout(keyFrame, 2000);
});
$(ME).find('#keyframe').on("error", function(){
  $(this).prop('src', me.prefix+'app/asset/camera/no_signal.jpg');
});
