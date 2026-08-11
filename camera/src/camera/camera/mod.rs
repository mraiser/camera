// This file is auto-generated and managed by the flowlang build script.
use flowlang::rustcmd::Transform;
pub mod events_page;
pub mod live;
pub mod stop_recording;
pub mod status;
pub mod start_recording;
pub mod snapshot;
pub mod settings;
pub mod remove_old;
pub mod play;
pub mod on_capture;
pub mod last_keyframe;
pub mod keyframes;
pub mod keyframe;
pub mod init;
pub mod events;
pub mod available;
pub fn cmdinit(cmds: &mut Vec<(String, Transform, String)>) {
    cmds.push(("vjpitv1843dfcd472td7".to_string(), available::execute, "".to_string()));
    cmds.push(("vyywsl18495656547m98".to_string(), events::execute, "".to_string()));
    cmds.push(("lpxgsi18439f44808s89".to_string(), init::execute, "".to_string()));
    cmds.push(("uypumz184f29704cbs39".to_string(), keyframe::execute, "".to_string()));
    cmds.push(("zvlkyu184b51974e2qa".to_string(), keyframes::execute, "".to_string()));
    cmds.push(("kwpmxu1844e0f248am45".to_string(), last_keyframe::execute, "".to_string()));
    cmds.push(("twnymk1844db44f3fve4".to_string(), on_capture::execute, "".to_string()));
    cmds.push(("guwsgi184b8daca10re".to_string(), play::execute, "".to_string()));
    cmds.push(("vgxvrr184f385730fy1da".to_string(), remove_old::execute, "".to_string()));
    cmds.push(("mgyrou18447eaa6c2l78e".to_string(), settings::execute, "".to_string()));
    cmds.push(("suqirv184446104d3x55".to_string(), snapshot::execute, "".to_string()));
    cmds.push(("lspkqt18439ff8220ta5".to_string(), start_recording::execute, "".to_string()));
    cmds.push(("norphu18452a74484p2ed".to_string(), status::execute, "".to_string()));
    cmds.push(("lkiphi1844d32b0f5z24".to_string(), stop_recording::execute, "".to_string()));
    cmds.push(("sgmxup1869485c680qaa".to_string(), live::execute, "".to_string()));
    cmds.push(("xuxzml19ff1f4faeau1".to_string(), events_page::execute, "".to_string()));
}
