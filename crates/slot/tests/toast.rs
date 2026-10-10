mod common;

use std::sync::atomic::Ordering;

use common::{app_playing_in, app_playing_with, tmp_root_with_carts, FaultySnapshot};
use slot_input::Action;
use slot_ui::{Toast, HUD_MS};

#[test]
fn a_save_hotkey_says_state_saved() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::SaveState);
    assert_eq!(a.toast(), Some(Toast::StateSaved));
}

#[test]
fn a_load_says_state_loaded() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::SaveState);
    a.apply(Action::LoadState);
    assert_eq!(a.toast(), Some(Toast::StateLoaded));
}

#[test]
fn a_state_the_core_rejects_says_load_failed_and_keeps_the_last_undo() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let core = FaultySnapshot::default();
    let mut a = app_playing_with(d.path(), "Emerald", core.boxed());
    a.apply(Action::SaveState);
    core.rejects.store(true, Ordering::Relaxed);
    a.apply(Action::LoadState);
    assert_eq!(a.toast(), Some(Toast::LoadFailed));
    assert_eq!(
        a.undo_label(),
        Some("undo save"),
        "a load that never happened replaced the save's undo"
    );
}

#[test]
fn a_refused_load_says_nothing() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply(Action::LoadState);
    assert_eq!(a.toast(), None, "it announced a load with an empty ring");
    assert!(a.refusal_active(a.now()), "and it did not shake either");
}

#[test]
fn a_toast_goes_away_on_its_own() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let mut a = app_playing_in(d.path(), "Emerald");
    a.apply_at(Action::SaveState, 1_000);
    assert_eq!(a.toast(), Some(Toast::StateSaved));
    a.tick_ms(1_000 + HUD_MS);
    assert_eq!(a.toast(), None);
}

#[test]
fn a_load_on_a_wedged_core_says_the_core_is_not_responding() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let core = FaultySnapshot::default();
    let mut a = app_playing_with(d.path(), "Emerald", core.boxed());
    a.apply(Action::SaveState);
    core.wedged.store(true, Ordering::Relaxed);
    let loads = core.loads.load(Ordering::Relaxed);
    a.apply(Action::LoadState);
    assert_eq!(a.toast(), Some(Toast::CoreNotResponding));
    assert_eq!(
        core.loads.load(Ordering::Relaxed),
        loads,
        "it waited on a load after the core had already stopped answering"
    );
}

#[test]
fn a_save_on_a_wedged_core_says_the_core_is_not_responding() {
    let d = tmp_root_with_carts(&["Emerald"]);
    let core = FaultySnapshot::default();
    let mut a = app_playing_with(d.path(), "Emerald", core.boxed());
    core.wedged.store(true, Ordering::Relaxed);
    a.apply(Action::SaveState);
    assert_eq!(a.toast(), Some(Toast::CoreNotResponding));
}
