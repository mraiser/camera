var monthnames = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];

var me = this; 
var ME = $('#'+me.UUID)[0];

me.ready = function(){
  if (document.body.api && document.body.api.ui) {
    me.uiReady(document.body.api.ui);
  }
  else {
    installControl('#dvr-ui', 'app', 'ui', function(ui) {}, {});
  }
};

me.uiReady = function(ui){
  me.ui = ui;
  ui.initNavbar(ME);
  ui.initSliders(ME);
  if (!ME.DATA.peer) me.prefix = "../";
  else me.prefix = "../peer/remote/"+ME.DATA.peer+"/";
  me.build();
};

me.build = function(){
  json(me.prefix+'camera/status', null, function(result){
    console.log(result);
    me.status = result.data;
    var el = $(ME).find('.img-wrap');
    adjustSizes();
    result.data.peer = ME.DATA.peer;
    installControl(el[0], 'camera', 'live', function(api){
      me.live = api;
    }, result.data);
  });

  $(ME).find('.tab-dvr').click(function(){
    renderDvrTab();
    $(ME).find('.video-wrap').css('display', 'none');
    var el = $(ME).find('.video-wrap-video');
    el[0].pause();
    el[0].removeAttribute('src');
    el[0].load();
    adjustSizes();
  });
  
  $(ME).find('.tab-live').click(function(){
    me.live.pause = false;
    $(ME).find('.snap-wrap').css('display', 'none');
    $(ME).find('.video-wrap').css('display', 'none');
    var el = $(ME).find('.video-wrap-video');
    el[0].pause();
    el[0].removeAttribute('src');
    el[0].load();
    adjustSizes();
  });

  $(ME).find('.tab-events').click(function(){
    $(ME).find('.snap-wrap').css('display', 'none');
    var div = $(ME).find('.event-images');
    div.empty();
    var render = function(result){
      // C4: the tab says what's going on - motion off, or on with nothing yet
      var list = (result && result.data) ? result.data : [];
      var msg = $(ME).find('.events-message');
      if (me.status && !me.status.motion) msg.text('Motion detection is off. Turn it on in ⚙ Settings.').css('display', 'block');
      else if (!list.length) msg.text('No motion events yet.').css('display', 'block');
      else msg.css('display', 'none');
      $(ME).find('.event-images').css('display', list.length ? 'block' : 'none');
      if (!result.data) return;
      result.data.sort(function(a,b){
        return a.time - b.time;
      });
      for (var i in result.data) {
        var d = result.data[i];
        var date = new Date(d.time);
        var start = d.time - 2000;
        var stop = d.frames[d.frames.length - 1].time + 2000;
        var el = $('<div class="thumbnail-wrap" data-start="'+start+'" data-stop="'+stop+'"><img class="thumbnail" src="'+me.prefix+'camera'+d.jpg+'"><div class="thumbnail-timestamp">'+parseTime(date)+'</div></div>');
        el.click(showClip);
        div.append(el);
      }
      adjustSizes();
    };
    // newest 500 via events_page; an older host (over the peer proxy)
    // lacks it, so fall back to the unpaginated events command
    json(me.prefix+'camera/events_page', "since=0&limit=500", function(result){
      if (result.status == 'ok' && result.data) render(result);
      else json(me.prefix+'camera/events', null, render);
    });
  });
  
  $(ME).find('.snap-wrap-img').on("error", function(){
    $(this).prop('src', '../app/asset/camera/no_signal.jpg');
  });
};

function buildThumbs(){
  if (!me.dvrtime || isNaN(me.dvrtime)) { return; }
  var start = me.dvrstart = me.dvrtime;
  var len = parseInt($(ME).find('.durationslider').val()) * 1000;
  $(ME).find('.durationslidervalue').val(parseLen(len/1000));
  var stop = me.dvrstop = start + len;
  var next = start;
  var div = $(ME).find('.dvr-images');
  div.empty();
  while (next <= stop) {
    var el = $('<div class="thumbnail-wrap" data-time="'+next+'"><img class="thumbnail" src="'+me.prefix+'camera/keyframe/kf-'+next+'.jpg?timestamp='+next+'"><div class="thumbnail-timestamp">'+parseTime(new Date(next))+'</div></div>');
    el.click(function(){ 
      me.loadSnap($(this).data('time'), function(){});
    });
    div.append(el);
    next += 2000;
  }
  $(ME).find('.download_selected').prop('href', me.prefix+'camera/play/'+start+'_'+stop+'.mp4');
}

// C4: the Duration readout follows the drag; thumbnails rebuild on release
$(ME).find('.durationslider').on('input', function(){
  $(ME).find('.durationslidervalue').val(parseLen(parseInt($(this).val())));
});
$(ME).find('.durationslider').on('change', buildThumbs);

function showClip(e) {
  var start = me.dvrtime = $(this).data('start');
  var stop = $(this).data('stop');
  play(start, stop);
}

function play(start, stop) {
  $(ME).find('.snap-wrap').css('display', 'none');
  var url = me.prefix+'camera/play/'+start+'_'+stop+'.mp4';
  me.live.pause = true;
  var el = $(ME).find('.video-wrap-video');
  el[0].pause();
  el[0].removeAttribute('src');
  el[0].load();
  el[0].src = url;
  
  // Wait for video metadata to load before showing and resizing
  el.off('loadedmetadata');
  el.on('loadedmetadata', function() {
    $(ME).find('.video-wrap').css('display', 'block');
    adjustSizes();
    $(ME).find('.live-video').resize();
  });
}

$(ME).find('.dvrplaybutton').click(function(){
  if (!(me.dvrstart > 0)) return;
  play(me.dvrstart, me.dvrstop);
});

// C4: the DVR tab says what's going on - off, empty, or the picker
function renderDvrTab(){
  var msg = $(ME).find('.dvr-message');
  var ctl = $(ME).find('.dvr-controls');
  if (!me.status || !me.status.dvr) {
    msg.text('DVR is off. Turn it on in ⚙ Settings.').css('display', 'block');
    $(ME).find('.dvr-images').css('display', 'none');
    ctl.css('display', 'none');
    return;
  }
  var empty = !me.status.first || !me.status.last;
  ctl.css('display', 'block');
  $(ME).find('.dvr-pickers, .dvr-duration').css('display', empty ? 'none' : 'inline-block');
  $(ME).find('.dvrplaybutton, .dvrdownloadbutton').prop('disabled', empty);
  $(ME).find('.dvr-images').css('display', empty ? 'none' : 'block');
  if (empty) {
    msg.text('No recordings yet.').css('display', 'block');
    $(ME).find('.download_selected').removeAttr('href');
  }
  else {
    msg.css('display', 'none');
    buildTimePicker();
  }
}

// the live control announces fresh status when the settings dialog closes
$(ME).on('camera-status', function(e, status){
  if (status) me.status = status;
  adjustSizes();
  if ($(ME).find('.navbar-tab.selected').data('id') == 'cam-dvr') renderDvrTab();
});

me.loadSnap = function(time, cb){
  if (!time || isNaN(time)) { if (cb) cb(); return; }
  var url = me.prefix+'camera/keyframe/snap-'+time+'.jpg?timestamp='+time;
  $(ME).find('.snap-wrap-img').prop('src', url);
  $(ME).find('.snap-wrap').css('display', 'block');
  if (cb) cb();
};

function adjustSizes(){
  // C3: a fit before camera/status answers is a no-op, never an error
  if (!me.status) return;
  var res = me.status.resolution;
  if (typeof res == "string") res = res.split("x");
  else if (!res) res = ['1920','1080'];
  var resw = parseInt(res[0]);
  var resh = parseInt(res[1]);
  if (!(resw > 0) || !(resh > 0)) { resw = 1920; resh = 1080; }
  // rotation turns the frames portrait; the picture box follows them
  var rot = parseInt(me.status.rotation);
  if (rot == 90 || rot == 270) { var t = resw; resw = resh; resh = t; }
  // pure CSS does the fit: full tile width at the picture's own ratio,
  // whatever order the first fit and the live install happen in
  $(ME).find('.picture-wrap').css('aspect-ratio', resw + ' / ' + resh);
  // C4: the tabs always show; the DVR tab itself says when DVR is off
  $(ME).find('.dvr-navbar').css('display', 'block');
}

function updateSlide(){
  var year = $(ME).find('.dvryearselect').find('select').val();
  var month = $(ME).find('.dvrmonthselect').find('select').val();
  var date = $(ME).find('.dvrdateselect').find('select').val();
  var hour = $(ME).find('.dvrhourselect').find('select').val();
  var min = $(ME).find('.dvrminselect').find('select').val();
  var sec = $(ME).find('.dvrsecselect').find('select').val();

  var nudate = new Date(year, month, date, hour, Number(min), Number(sec), 0);
  me.dvrtime = nudate.getTime();
  buildTimePicker();
  
  var p = (me.dvrtime - me.status.first) / (me.status.last - me.status.first);
  $(ME).find('.datetimeslider').val(p*100).trigger("input");
}

function buildTimePicker(){
  // an empty archive has no first/last - a picker built from them would
  // emit timestamp=NaN requests forever
  if (!me.status.first || !me.status.last) { return; }
  var start = new Date(me.status.first);
  var stop = new Date(me.status.last);

  if (!me.dvrtime) me.dvrtime = me.status.last;
  else if (me.dvrtime < me.status.first) me.dvrtime = me.status.first;
  else if (me.dvrtime > me.status.last) me.dvrtime = me.status.last;
  var current = new Date(me.dvrtime);

  me.live.pause = true;
  me.loadSnap(me.dvrtime, function(){
    // FIXME
    $(ME).find('.timestamp').css('color', 'white');
  }, 0, true);
  
  var startyear = start.getFullYear();
  var stopyear = stop.getFullYear();
  var currentyear = current.getFullYear();
  var years = [];
  for (var i=startyear; i<stopyear+1; i++) years.push(i);
  var d = {
    "list": years,
    "value": ""+currentyear,
    "label": "Year",
    "cb": updateSlide
  }
  var el = $(ME).find('.dvryearselect');
  installControl(el[0], 'app', 'select', function(api){}, d);
  
  var currentmonth = current.getMonth();
  var months = [];
  for (var i=0; i<12; i++){
    var mstart = new Date(currentyear, i, 1, 0, 0, 0, 0);
    var mend = new Date(currentyear, i+1, 1, 0, 0, 0, -1);
    var mst = mstart.getTime();
    var met = mend.getTime();
    if (met>me.status.first && mst<me.status.last){
      d = {
        "id": ""+i,
        "name": monthnames[i].substring(0, 3)
      };
      months.push(d);
    }
  }
  d = {
    "list": months,
    "value": ""+currentmonth,
    "label": "Month",
    "cb": updateSlide
  };
  el = $(ME).find('.dvrmonthselect');
  installControl(el[0], 'app', 'select', function(api){}, d);
  
  var currentdate = current.getDate();
  var sdate = mstart.getDate();
  var edate = mend.getDate()+1;
  var dates = [];
  for (var i=sdate;i<edate;i++){
    var dstart = new Date(currentyear, currentmonth, i, 0, 0, 0, 0);
    var dend = new Date(currentyear, currentmonth, i+1, 0, 0, 0, -1);
    var dst = dstart.getTime();
    var det = dend.getTime();
    if (det>me.status.first && dst<me.status.last) {
      dates.push(""+i);
    }
  }
  d = {
    "list": dates,
    "value": ""+currentdate,
    "label": "Date",
    "cb": updateSlide
  };
  el = $(ME).find('.dvrdateselect');
  installControl(el[0], 'app', 'select', function(api){}, d);
  
  var currenthour = current.getHours();
  var hours = [];
  for (var i=0; i<24; i++){
    var dstart = new Date(currentyear, currentmonth, currentdate, i, 0, 0, 0);
    var dend = new Date(currentyear, currentmonth, currentdate, i+1, 0, 0, -1);
    var dst = dstart.getTime();
    var det = dend.getTime();
    if (det>me.status.first && dst<me.status.last) {
      hours.push(""+i);
    }
  }
  d = {
    "list": hours,
    "value": ""+currenthour,
    "label": "Hour",
    "cb": updateSlide
  };
  el = $(ME).find('.dvrhourselect');
  installControl(el[0], 'app', 'select', function(api){}, d);
  
  var currentmin = current.getMinutes();
  var minutes = [];
  for (var i=0; i<60; i++){
    var dstart = new Date(currentyear, currentmonth, currentdate, currenthour, i, 0, 0);
    var dend = new Date(currentyear, currentmonth, currentdate, currenthour, i+1, 0, -1);
    var dst = dstart.getTime();
    var det = dend.getTime();
    if (det>me.status.first && dst<me.status.last) {
      minutes.push(""+padZero(i));
    }
  }
  d = {
    "list": minutes,
    "value": ""+padZero(currentmin),
    "label": "Minute",
    "cb": updateSlide
  };
  el = $(ME).find('.dvrminselect');
  installControl(el[0], 'app', 'select', function(api){}, d);
  
  var currentsec = current.getSeconds();
  var seconds = [];
  for (var i=0; i<60; i++){
    var dstart = new Date(currentyear, currentmonth, currentdate, currenthour, currentmin, i, 0);
    var dend = new Date(currentyear, currentmonth, currentdate, currenthour, currentmin, i+1, -1);
    var dst = dstart.getTime();
    var det = dend.getTime();
    if (det>me.status.first && dst<me.status.last) {
      seconds.push(""+padZero(i));
    }
  }
  d = {
    "list": seconds,
    "value": ""+padZero(currentsec),
    "label": "Second",
    "cb": updateSlide
  };
  el = $(ME).find('.dvrsecselect');
  installControl(el[0], 'app', 'select', function(api){
    adjustSizes();
  }, d);
 
  buildThumbs();
};



function parseLen(len) {
  let sec = padZero(len % 60,1);
  let min = parseInt(len/60);
  return min+":"+sec;
}

function parseTime(d){
  return parseDate(d)+':'+padZero(d.getSeconds());
}

// C3: refit from the tile's own box (dragging the Peers panel never fires
// window.resize, and window handlers piled up one per remount). One
// observer per mount; it drops itself when the tile leaves the page.
me.sizer = new ResizeObserver(function(){
  if (!ME.isConnected) { me.sizer.disconnect(); return; }
  adjustSizes();
});
me.sizer.observe(ME);
