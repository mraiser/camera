var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (!ME.DATA.peer) me.prefix = "../";
  else me.prefix = "../peer/remote/"+ME.DATA.peer+"/";
  
  document.body.api.ui.initSliders(ME);

  // C9: "value / max" beside each motion slider, live while dragging. Bound
  // before the load trigger('input') below so the first paint is correct.
  function bindSliderValue(sel, out){
    var s = $(ME).find(sel), o = $(ME).find(out);
    var paint = function(){ o.text(s.val() + ' / ' + s.attr('max')); };
    s.on('input', paint);
    paint();
  }
  bindSliderValue('.rvs-sense', '.rvs-sense-value');
  bindSliderValue('.rvs-noise', '.rvs-noise-value');

  // C6: the dialog is mounted fresh on every open, so every open asks the
  // camera for its current settings; both sections always show
  json(me.prefix+'camera/settings', null, function(result) {
    me.settings = (result && result.data) ? result.data : {};
    $(ME).find('.rvs-sense').val(me.settings.motion_sensitivity).trigger("input");
    $(ME).find('.rvs-noise').val(me.settings.motion_noise_cancel).trigger("input");
    $(ME).find('.dvr-switch').prop('checked', !!me.settings.dvr);
    $(ME).find('.motion-switch').prop('checked', !!me.settings.motion);
    $(ME).find('.storagedir').val(me.settings.storage);
    $(ME).find('.select-fps').val(me.settings.framerate);
    showRecording();
    var cs = $.extend({}, me.settings, { peer: ME.DATA.peer });
    installControl($(ME).find('.camera-settings-go-here')[0], 'camera', 'camera_settings', function(api){}, cs);
  });
};

// C6: flipping DVR shows its effect at once, before Save - the recording
// fields dim, and the Camera section says a save restarts recording
// (settings.rs stops and restarts the capture on every save with DVR on)
function showRecording(){
  var on = $(ME).find('.dvr-switch').prop('checked');
  var fields = $(ME).find('.rec-fields');
  fields.toggleClass('dimmed', !on);
  fields.find('input, select').prop('disabled', !on);
  $(ME).find('.cam-restart-note').css('display', on ? 'block' : 'none');
  $(ME).find('.motion-settings').css('display', $(ME).find('.motion-switch').prop('checked') ? 'block' : 'none');
}
$(ME).find('.dvr-switch, .motion-switch').on('change', showRecording);

// C6: a failed save says why in the footer and keeps the dialog open. The
// reply can fail at the top (peer unreachable) or inside (the command's own
// {status:err} wrapped as data)
function saveError(r){
  if (!r) return "Couldn't save: no reply from the camera.";
  if (r.status != 'ok') return "Couldn't save: " + (r.msg || "the camera didn't accept the settings.");
  if (r.data && r.data.status == 'err') return "Couldn't save: " + (r.data.msg || "the camera didn't accept the settings.");
  return null;
}

$(ME).find('.save-camera-settings').click(function(){
  var btn = $(this);
  var d = {};
  // only real values: a field that isn't there (no camera) must not
  // overwrite the stored setting with null
  function put(k, v){ if (v !== undefined && v !== null && v !== '') d[k] = v; }
  put('camera', $(ME).find('#select-camera').val());
  put('device', $(ME).find('#select-device').val());
  put('format', $(ME).find('#select-format').val());
  put('resolution', $(ME).find('#select-resolution').val());
  put('rotation', $(ME).find('#select-rotation').val());
  put('storage', $.trim($(ME).find('.storagedir').val() || ''));
  put('framerate', $(ME).find('.select-fps').val());
  put('motion_sensitivity', $(ME).find('.rvs-sense').val());
  put('motion_noise_cancel', $(ME).find('.rvs-noise').val());
  d.dvr = $(ME).find('.dvr-switch').prop('checked');
  d.motion = $(ME).find('.motion-switch').prop('checked');
  d.recording = d.dvr;
  var errline = $(ME).find('.save-error').css('display', 'none');
  btn.prop('disabled', true);
  json(me.prefix+'camera/settings', "settings="+encodeURIComponent(JSON.stringify(d)), function(result) {
    btn.prop('disabled', false);
    var err = saveError(result);
    if (err) { errline.text(err).css('display', 'block'); return; }
    $(ME).closest('.settings-popup').find('.close-camera-settings').click();
  });
});