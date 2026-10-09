use slot_store::{move_config, read_slot_state};

const FILES: [(&str, &str); 5] = [
    ("slot.state", "connector.state"),
    ("selected_core.ini", "selected_core.ini"),
    ("video_mode.ini", "video_mode.ini"),
    ("cart_shell.ini", "cart_shell.ini"),
    ("theme.txt", "theme.txt"),
];

#[test]
fn the_players_files_move_from_system_to_config() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("System")).unwrap();
    for (f, _) in FILES {
        std::fs::write(d.path().join("System").join(f), f).unwrap();
    }
    std::fs::write(d.path().join("System/connector"), "binary").unwrap();

    move_config(d.path()).unwrap();

    for (f, to) in FILES {
        assert_eq!(
            std::fs::read_to_string(d.path().join("Config").join(to)).unwrap(),
            f
        );
        assert!(
            !d.path().join("System").join(f).exists(),
            "{f} was left behind"
        );
    }
    assert!(
        d.path().join("System/connector").exists(),
        "a shipped file moved"
    );
}

#[test]
fn a_file_already_in_config_is_not_overwritten() {
    let d = tempfile::tempdir().unwrap();
    for dir in ["System", "Config"] {
        std::fs::create_dir_all(d.path().join(dir)).unwrap();
    }
    std::fs::write(d.path().join("System/selected_core.ini"), "old").unwrap();
    std::fs::write(d.path().join("Config/selected_core.ini"), "new").unwrap();

    move_config(d.path()).unwrap();

    assert_eq!(
        std::fs::read_to_string(d.path().join("Config/selected_core.ini")).unwrap(),
        "new"
    );
}

#[test]
fn settings_written_before_the_move_are_still_read_after_it() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("System")).unwrap();
    std::fs::write(
        d.path().join("System/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=0\nclock_set=1\nutc_offset_min=-240\n",
    )
    .unwrap();

    move_config(d.path()).unwrap();

    let s = read_slot_state(d.path());
    assert_eq!(
        (s.cart.as_deref(), s.volume, s.utc_offset_min),
        (Some("Emerald"), 40, -240)
    );
}

#[test]
fn slot_settings_already_in_config_are_renamed_and_kept() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=40\nmuted=0\nclock_set=1\nutc_offset_min=-240\n",
    )
    .unwrap();

    move_config(d.path()).unwrap();

    assert!(!d.path().join("Config/slot.state").exists());
    let s = read_slot_state(d.path());
    assert_eq!(
        (s.cart.as_deref(), s.volume, s.utc_offset_min),
        (Some("Emerald"), 40, -240)
    );
}

#[test]
fn connector_settings_are_not_overwritten_by_old_slot_ones() {
    let d = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(d.path().join("Config")).unwrap();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=10\nmuted=0\nclock_set=1\nutc_offset_min=0\n",
    )
    .unwrap();
    std::fs::write(
        d.path().join("Config/connector.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=70\nmuted=0\nclock_set=1\nutc_offset_min=0\n",
    )
    .unwrap();

    move_config(d.path()).unwrap();

    assert_eq!(read_slot_state(d.path()).volume, 70);
}

#[test]
fn settings_in_config_win_over_older_ones_left_in_system() {
    let d = tempfile::tempdir().unwrap();
    for dir in ["System", "Config"] {
        std::fs::create_dir_all(d.path().join(dir)).unwrap();
    }
    std::fs::write(
        d.path().join("System/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=10\nmuted=0\nclock_set=1\nutc_offset_min=0\n",
    )
    .unwrap();
    std::fs::write(
        d.path().join("Config/slot.state"),
        "cart=Emerald\nbrightness=3\nblue_light=1\nvolume=70\nmuted=0\nclock_set=1\nutc_offset_min=0\n",
    )
    .unwrap();

    move_config(d.path()).unwrap();
    move_config(d.path()).unwrap();

    assert_eq!(read_slot_state(d.path()).volume, 70);
    assert!(!d.path().join("Config/slot.state").exists());
}
