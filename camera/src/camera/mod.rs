// This file is auto-generated and managed by the flowlang build script.
use flowlang::rustcmd::Transform;
pub mod camera;
pub fn cmdinit(cmds: &mut Vec<(String, Transform, String)>) {
    camera::cmdinit(cmds);
}
