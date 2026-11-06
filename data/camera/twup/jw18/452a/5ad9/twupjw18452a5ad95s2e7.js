var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  me.status = ME.DATA
  if (!ME.DATA.peer) me.prefix = "../";
  else me.prefix = "../peer/remote/"+ME.DATA.peer+"/";
  if (ME.DATA.controls == false) $(ME).find('.live-img-ctrls').css('display', 'none');
  // step 0a: when nothing on this UA can go full screen (iPhone Safari has
  // no element fullscreen and no video webkitEnterFullscreen), hide the button
  var fsprobe = $(ME).find('.live-wrap')[0];
  if (!fsprobe.requestFullscreen && !fsprobe.webkitRequestFullscreen && !$(ME).find('.live-video')[0].webkitEnterFullscreen) {
    $(ME).find('.fullscreen-button').css('display', 'none');
  }
  if (me.status.device) start();
  else {
    json(me.prefix+'camera/status', null, function(result){
      var peer = ME.DATA.peer;
      if (result.data) {
        me.status = ME.DATA = result.data;
        me.status.peer = peer;
        start();
      }
      else setTimeout(me.ready, 1000);
    });
  }
};

// Live view, best transport first: when the DVR capture stream is running
// and the browser has MSE, play the camera/live fragmented-MP4 stream in a
// video element; on any failure fall back to the classic ~2s JPEG polling.
function start(){
  showConnecting(true);
  if (me.status.dvr && window.MediaSource && !me.msefailed && !me.pause) startMSE();
  else next();
}

// C4: "Connecting…" on the tile's own background while the stream starts;
// it goes away when the first picture (or the no-signal card) arrives
function showConnecting(on){
  $(ME).find('.live-connecting').css('display', on ? 'block' : 'none');
}

// The exact codec rides in the stream: read it from the codec config box
// of the init segment (first fetch chunk) so any camera profile works
// without guessing - avcC for H.264, vpcC for VP9.
function typeFromInit(chunk){
  var hex = function(b){ return ('0'+b.toString(16)).slice(-2); };
  var dec2 = function(b){ return ('0'+b.toString(10)).slice(-2); };
  for (var i=0; i+12 < chunk.length; i++) {
    if (chunk[i]==0x61 && chunk[i+1]==0x76 && chunk[i+2]==0x63 && chunk[i+3]==0x43) { // 'avcC'
      return 'video/mp4; codecs="avc1.'+hex(chunk[i+5])+hex(chunk[i+6])+hex(chunk[i+7])+'"';
    }
    if (chunk[i]==0x76 && chunk[i+1]==0x70 && chunk[i+2]==0x63 && chunk[i+3]==0x43) { // 'vpcC'
      var profile = chunk[i+8], level = chunk[i+9], depth = chunk[i+10] >> 4;
      return 'video/mp4; codecs="vp09.'+dec2(profile)+'.'+dec2(level)+'.'+dec2(depth)+'"';
    }
  }
  return 'video/mp4; codecs="avc1.42e01f"';
}

function startMSE(){
  var video = $(ME).find('.live-video')[0];
  var ms = new MediaSource();
  var aborter = new AbortController();
  var queue = [];
  var sb = null;
  var dead = false;

  function teardown(){
    if (dead) return;
    dead = true;
    try { aborter.abort(); } catch(e) {}
    try {
      if (video.src) { URL.revokeObjectURL(video.src); video.removeAttribute('src'); video.load(); }
    } catch(e) {}
    $(ME).find('.live-video').css('display', 'none');
    $(ME).find('.live-img').css('display', '');
  }
  me.stopMSE = teardown;

  function fail(){
    if (dead) return;
    teardown();
    me.msefailed = true;
    next();
  }

  function pump(){
    if (dead || !sb || sb.updating || !queue.length) return;
    try {
      if (video.buffered.length && video.buffered.end(video.buffered.length-1) - video.buffered.start(0) > 60) {
        sb.remove(video.buffered.start(0), video.buffered.end(video.buffered.length-1) - 15);
        return;
      }
      sb.appendBuffer(queue.shift());
    } catch(e) { fail(); }
  }

  ms.addEventListener('sourceopen', function(){
    fetch(me.prefix+'camera/live/live.mp4?t='+(new Date().getTime()), {signal: aborter.signal, credentials: 'same-origin'}).then(function(res){
      if (!res.ok || !res.body) return fail();
      var reader = res.body.getReader();
      function loop(){
        reader.read().then(function(r){
          if (dead) return;
          if (r.done) return fail();
          if (!sb) {
            var type = typeFromInit(r.value);
            if (!MediaSource.isTypeSupported(type)) type = 'video/mp4; codecs="avc1.42e01f"';
            try { sb = ms.addSourceBuffer(type); } catch(e) { return fail(); }
            sb.addEventListener('updateend', pump);
            sb.addEventListener('error', fail);
          }
          queue.push(r.value);
          pump();
          loop();
        }, function(){ if (!dead) fail(); });
      }
      loop();
    }, function(){ if (!dead) fail(); });
  });
  video.addEventListener('error', fail);

  $(ME).find('.live-img').css('display', 'none');
  $(ME).find('.live-video').css('display', 'block');
  $(video).off('playing.conn').on('playing.conn', function(){ showConnecting(false); });
  video.muted = true;
  video.src = URL.createObjectURL(ms);
  
  // Wait for video metadata to load before triggering resize
  $(video).off('loadedmetadata');
  $(video).on('loadedmetadata', function() {
    $(ME).find('.live-video').resize();
  });
  
  if (video.play) video.play().catch(function(){});

  // stall watchdog: if nothing has buffered within 12s (headerless
  // mid-stream attach on an old capture process, proxy hiccup, ...)
  // degrade to polling instead of showing a black video forever
  setTimeout(function(){
    if (!dead && !(video.buffered && video.buffered.length)) fail();
  }, 12000);

  // stay near the live edge; honor pause (the DVR tabs set me.pause)
  var watcher = setInterval(function(){
    if (dead) { clearInterval(watcher); return; }
    if (me.pause) {
      clearInterval(watcher);
      teardown();
      watchPause();
      return;
    }
    try {
      if (video.buffered.length) {
        var edge = video.buffered.end(video.buffered.length-1);
        if (edge - video.currentTime > 5) video.currentTime = edge - 0.5;
      }
    } catch(e) {}
  }, 2000);
}

function watchPause(){
  if (me.pause) { setTimeout(watchPause, 500); return; }
  start();
}

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
    // the fallback assignment below stores the ARRAY back into
    // me.status.resolution, so the next poll reads an array - only
    // strings split (same guard adjustSizes always had)
    if (typeof res == "string") res = res.split("x");
    else if (!res) res = me.status.resolution = ['1920','1080'];
    $(ME).find('.live-img').prop("src", me.prefix+"camera/snapshot/x"+(q++)+".jpg?device="+encodeURIComponent(dev)+"&format="+encodeURIComponent(format)+"&width="+res[0]+"&height="+res[1]+"&rot="+rot);
  }
}

$(ME).find('.live-img').on("load", function(){
  showConnecting(false);
  setTimeout(next, 2000);
});

$(ME).find('.live-img').on("error", function(){
  $(this).prop('src', '../app/asset/camera/no_signal.jpg');
});

$(ME).find('.dvr-settings-button').click(function(e){
  me.pause = true;
  if (me.stopMSE) me.stopMSE();
  e.selector = $(ME).find(".settings-popup")[0];
  e.closeselector = $(ME).find(".close-camera-settings")[0];
  e.modal = true;
  e.close = function() { 
    json(me.prefix+'camera/status', null, function(result){
      me.status = ME.DATA = result.data;
      me.pause = false; 
      me.msefailed = false;
      start();
      // C4: the tabs stay; they say what is off. Tell the tile the new state.
      $(ME).closest('.dvr-wrap').find('.dvr-navbar').css('display', 'block');
      $(ME).trigger('camera-status', [result.data]);
    });
  };
  
  document.body.api.ui.popup(e, function(){
    me.status.peer = ME.DATA.peer;
    installControl($(ME).find(".settings-popup-inner")[0], "camera", "dvr_settings", function(){}, me.status);
  });
});

// C3: ⛶ fullscreens the whole picture area (the dvr control's aspect box,
// so DVR playback and snapshots ride along); standalone fallback is this
// control's own wrap. The UA stretches the fullscreen element to the
// screen and object-fit letterboxes the picture.
function fsTarget(){
  var pw = $(ME).closest('.picture-wrap');
  return pw.length ? pw[0] : $(ME).find('.live-wrap')[0];
}

// C3: icons swap ONLY on fullscreenchange, so every exit path (Esc, the
// exit button, the browser's own UI) restores them, and a refused request
// changes nothing because no event fires. Document-level listener: it
// unhooks itself once this mount leaves the page.
function fsSync(){
  if (!ME.isConnected) {
    document.removeEventListener('fullscreenchange', fsSync);
    document.removeEventListener('webkitfullscreenchange', fsSync);
    return;
  }
  var fs = document.fullscreenElement || document.webkitFullscreenElement;
  var active = !!fs && fs.contains(ME);
  $(ME).find('.fullscreen-button').css('display', active ? 'none' : 'inline-block');
  $(ME).find('.fullscreen-exit-button').css('display', active ? 'inline-block' : 'none');
  $(ME).find('.dvr-settings-button').css('display', active ? 'none' : 'inline-block');
}
document.addEventListener('fullscreenchange', fsSync);
document.addEventListener('webkitfullscreenchange', fsSync);

$(ME).find('.fullscreen-button').click(function(){
  var elem = fsTarget();
  var p = null;
  try {
    if (elem.requestFullscreen) p = elem.requestFullscreen();
    else if (elem.webkitRequestFullscreen) p = elem.webkitRequestFullscreen(); /* Safari */
    else { /* iPhone Safari: no element full screen - the video itself can */
      var v = fsVideo();
      if (v) v.webkitEnterFullscreen();
    }
  } catch(e) {}
  if (p && p.catch) p.catch(function(){});
});

// the video showing in this picture area (DVR playback or live) - iPhone
// Safari offers video-only full screen through webkitEnterFullscreen
function fsVideo(){
  var pw = $(ME).closest('.picture-wrap');
  var scope = pw.length ? pw[0] : ME;
  var vids = scope.querySelectorAll('video');
  var fallback = null;
  for (var i = 0; i < vids.length; i++) {
    if (!vids[i].webkitEnterFullscreen) continue;
    if (!fallback) fallback = vids[i];
    if (vids[i].offsetParent !== null) return vids[i];
  }
  return fallback;
}

$(ME).find('.fullscreen-exit-button').click(function(){
  try {
    if (document.exitFullscreen) document.exitFullscreen();
    else if (document.webkitExitFullscreen) document.webkitExitFullscreen(); /* Safari */
  } catch(e) {}
});