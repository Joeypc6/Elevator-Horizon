// `native_tooltip_render_pre`: while the guide is open, the screen behind it shows no tooltip over its pages (the
// guide draws its own items' tooltips itself).
fn kit_guide_hide_tip_elevatorid(cancel: &mut bool) {
    elevatorid_optional::if_kit!("elevatorid", {
    if crate::game::hz_kit_guide_elevatorid::is_open() {
        *cancel = true;
    }
} else { let _ = cancel; () })
}
