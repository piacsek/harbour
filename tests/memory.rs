use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use harbour::memory::{Memory, Remembered};

const ID: &str = "0f1e2d3c-4b5a-6978-8a9b-0c1d2e3f4a5b";
const OTHER_ID: &str = "9a8b7c6d-5e4f-4a3b-2c1d-0e9f8a7b6c5d";
const DAY: u64 = 24 * 60 * 60;

#[test]
fn a_kept_session_is_recalled() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));

    memory.keep(&remembered(ID, 0)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn the_file_is_named_after_the_session_id() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());

    Memory::new(&dir).keep(&remembered(ID, 0)).unwrap();

    assert_eq!(names(&dir), vec![format!("{ID}.json")]);
}

#[test]
fn the_folder_and_the_file_are_private() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());

    Memory::new(&dir).keep(&remembered(ID, 0)).unwrap();

    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&dir.join(format!("{ID}.json"))), 0o600);
}

#[test]
fn an_open_folder_is_made_private() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();

    Memory::new(&dir).keep(&remembered(ID, 0)).unwrap();

    assert_eq!(mode(&dir), 0o700);
}

#[test]
fn keeping_again_replaces_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    memory.keep(&remembered(ID, 0)).unwrap();

    memory.keep(&remembered(ID, 3)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 3)]);
}

#[test]
fn sessions_are_recalled_by_position() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    memory.keep(&remembered(ID, 1)).unwrap();
    memory.keep(&remembered(OTHER_ID, 0)).unwrap();

    let recalled = memory.recall();

    assert_eq!(recalled[0].session_id, OTHER_ID);
    assert_eq!(recalled[1].session_id, ID);
}

#[test]
fn equal_positions_are_ordered_by_id() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    memory.keep(&remembered(OTHER_ID, 0)).unwrap();
    memory.keep(&remembered(ID, 0)).unwrap();

    let recalled = memory.recall();

    assert_eq!(recalled[0].session_id, ID);
    assert_eq!(recalled[1].session_id, OTHER_ID);
}

#[test]
fn forgetting_removes_the_file() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();

    memory.forget(ID);

    assert_eq!(names(&dir), Vec::<String>::new());
}

#[test]
fn nothing_is_recalled_without_a_folder() {
    let scratch = tempfile::tempdir().unwrap();

    let recalled = Memory::new(&folder(scratch.path())).recall();

    assert_eq!(recalled, Vec::new());
}

#[test]
fn a_symlinked_folder_is_refused() {
    let scratch = tempfile::tempdir().unwrap();
    let elsewhere = scratch.path().join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    fs::create_dir_all(scratch.path().join("harbour")).unwrap();
    symlink(&elsewhere, folder(scratch.path())).unwrap();

    let kept = Memory::new(&folder(scratch.path())).keep(&remembered(ID, 0));

    assert!(kept.is_err());
    assert_eq!(names(&elsewhere), Vec::<String>::new());
}

#[test]
fn an_id_that_is_not_a_plain_name_is_not_kept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());

    let kept = Memory::new(&dir).keep(&remembered("../acme", 0));

    assert!(kept.is_err());
    assert!(!scratch.path().join("harbour/acme.json").exists());
}

#[test]
fn a_leftover_temporary_file_is_ignored() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(
        dir.join(format!(".{OTHER_ID}.1-2-3.tmp")),
        json(OTHER_ID, 1),
    )
    .unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn malformed_files_are_skipped_and_kept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let bad = dir.join(format!("{OTHER_ID}.json"));
    fs::write(&bad, "{\"version\":1,\"sessionId\":").unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
    assert!(bad.exists());
}

#[test]
fn a_file_whose_name_disagrees_with_its_id_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(dir.join(format!("{OTHER_ID}.json")), json(ID, 1)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn an_id_that_is_not_a_plain_name_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(dir.join("acme;rm.json"), json("acme;rm", 1)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_relative_directory_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(
        dir.join(format!("{OTHER_ID}.json")),
        format!(
            r#"{{"version":1,"sessionId":"{OTHER_ID}","directory":"acme-api","title":"Acme","position":1}}"#
        ),
    )
    .unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_wrong_field_type_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(
        dir.join(format!("{OTHER_ID}.json")),
        format!(
            r#"{{"version":1,"sessionId":"{OTHER_ID}","directory":"/home/acme","title":7,"position":1}}"#
        ),
    )
    .unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn an_oversized_file_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let title = "a".repeat(70 * 1024);
    fs::write(
        dir.join(format!("{OTHER_ID}.json")),
        format!(
            r#"{{"version":1,"sessionId":"{OTHER_ID}","directory":"/home/acme","title":"{title}","position":1}}"#
        ),
    )
    .unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_symlinked_file_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let outside = scratch.path().join("outside.json");
    fs::write(&outside, json(OTHER_ID, 1)).unwrap();
    symlink(&outside, dir.join(format!("{OTHER_ID}.json"))).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_folder_named_like_a_file_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::create_dir(dir.join(format!("{OTHER_ID}.json"))).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn an_unreadable_file_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let locked = dir.join(format!("{OTHER_ID}.json"));
    fs::write(&locked, json(OTHER_ID, 1)).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_file_from_a_newer_harbour_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(dir.join(format!("{OTHER_ID}.json")), newer(OTHER_ID)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_file_from_a_newer_harbour_is_never_overwritten() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let file = dir.join(format!("{ID}.json"));
    fs::write(&file, newer(ID)).unwrap();

    let kept = memory.keep(&remembered(ID, 4));

    assert!(kept.is_err());
    assert_eq!(fs::read_to_string(&file).unwrap(), newer(ID));
}

#[test]
fn a_file_from_a_newer_harbour_is_never_forgotten() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let file = dir.join(format!("{ID}.json"));
    fs::write(&file, newer(ID)).unwrap();

    memory.forget(ID);

    assert!(file.exists());
}

#[test]
fn forgetting_an_id_that_is_not_a_plain_name_removes_nothing() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let outside = scratch.path().join("harbour/acme.json");
    fs::create_dir_all(scratch.path().join("harbour")).unwrap();
    fs::write(&outside, "acme").unwrap();

    Memory::new(&dir).forget("../acme");

    assert!(outside.exists());
}

#[test]
fn unknown_fields_are_ignored() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{ID}.json")),
        format!(
            r#"{{"version":1,"sessionId":"{ID}","directory":"/home/acme/api","title":"Fix the flaky login test","position":0,"colour":"teal"}}"#
        ),
    )
    .unwrap();

    assert_eq!(Memory::new(&dir).recall(), vec![remembered(ID, 0)]);
}

#[test]
fn an_id_shaped_like_an_option_is_not_kept() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());

    let kept = Memory::new(&dir).keep(&remembered("--dangerously-skip-permissions", 0));

    assert!(kept.is_err());
    assert!(!dir.join("--dangerously-skip-permissions.json").exists());
}

#[test]
fn a_file_named_like_an_option_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(dir.join("-r.json"), json("-r", 1)).unwrap();
    fs::write(dir.join("--settings.json"), json("--settings", 2)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn an_id_that_is_not_shaped_like_a_claude_code_session_id_is_skipped() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let upper = OTHER_ID.to_uppercase();
    fs::write(dir.join(format!("{upper}.json")), json(&upper, 1)).unwrap();
    fs::write(dir.join("acme-1.json"), json("acme-1", 2)).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

#[test]
fn a_file_another_file_replaced_is_dropped_and_removed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    fs::write(
        dir.join(format!("{OTHER_ID}.json")),
        replacing(OTHER_ID, ID),
    )
    .unwrap();

    let recalled = memory.recall();

    assert_eq!(recalled, vec![remembered(OTHER_ID, 0)]);
    assert!(!dir.join(format!("{ID}.json")).exists());
}

#[test]
fn keeping_a_replacement_forgets_the_old_file() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();

    memory.replace(ID, &remembered(OTHER_ID, 0)).unwrap();

    assert_eq!(names(&dir), vec![format!("{OTHER_ID}.json")]);
}

#[test]
fn a_replacement_records_the_id_it_replaces() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);

    memory.replace(ID, &remembered(OTHER_ID, 0)).unwrap();

    let text = fs::read_to_string(dir.join(format!("{OTHER_ID}.json"))).unwrap();
    assert!(text.contains(&format!(r#""replaces":"{ID}""#)), "{text}");
}

#[test]
fn a_file_replacing_a_missing_one_is_recalled() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{OTHER_ID}.json")),
        replacing(OTHER_ID, ID),
    )
    .unwrap();

    assert_eq!(Memory::new(&dir).recall(), vec![remembered(OTHER_ID, 0)]);
}

#[test]
fn a_kept_file_is_held() {
    let scratch = tempfile::tempdir().unwrap();
    let memory = Memory::new(&folder(scratch.path()));
    memory.keep(&remembered(ID, 0)).unwrap();

    assert!(memory.holds(ID));
    assert!(!memory.holds(OTHER_ID));
}

#[test]
fn an_old_file_is_forgotten_as_stale() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    age(
        &dir.join(format!("{ID}.json")),
        Duration::from_secs(31 * DAY),
    );

    memory.forget_stale(ID);

    assert!(!dir.join(format!("{ID}.json")).exists());
}

#[test]
fn a_recent_file_is_not_forgotten_as_stale() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    age(
        &dir.join(format!("{ID}.json")),
        Duration::from_secs(29 * DAY),
    );

    memory.forget_stale(ID);

    assert!(dir.join(format!("{ID}.json")).exists());
}

#[test]
fn a_reserved_id_cannot_be_reserved_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let first = Memory::new(&dir).reserve(ID).unwrap();

    let second = Memory::new(&dir).reserve(ID).unwrap();

    assert!(first.is_some());
    assert!(second.is_none());
}

#[test]
fn a_released_id_can_be_reserved_again() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    drop(Memory::new(&dir).reserve(ID).unwrap());

    let again = Memory::new(&dir).reserve(ID).unwrap();

    assert!(again.is_some());
}

#[test]
fn another_id_is_reserved_on_its_own() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let _first = Memory::new(&dir).reserve(ID).unwrap();

    let other = Memory::new(&dir).reserve(OTHER_ID).unwrap();

    assert!(other.is_some());
}

#[test]
fn reserved_ids_are_listed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let _held = Memory::new(&dir).reserve(ID).unwrap();

    let reserved = Memory::new(&dir).reserved();

    assert!(reserved.contains(ID));
    assert!(!reserved.contains(OTHER_ID));
}

#[test]
fn a_dead_reservation_is_swept_and_not_listed() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    fs::create_dir_all(&dir).unwrap();
    let dead = dir.join(format!("{ID}.1-2-3.lock"));
    fs::write(&dead, "").unwrap();

    let reserved = Memory::new(&dir).reserve(OTHER_ID).unwrap();

    assert!(reserved.is_some());
    assert!(!Memory::new(&dir).reserved().contains(ID));
    assert!(!dead.exists());
}

#[test]
fn a_reservation_is_private() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let _held = Memory::new(&dir).reserve(ID).unwrap();

    let lock = names(&dir)
        .into_iter()
        .find(|name| name.ends_with(".lock"))
        .unwrap();

    assert_eq!(mode(&dir.join(lock)), 0o600);
}

#[test]
fn a_reservation_is_not_recalled() {
    let scratch = tempfile::tempdir().unwrap();
    let dir = folder(scratch.path());
    let memory = Memory::new(&dir);
    memory.keep(&remembered(ID, 0)).unwrap();
    let _held = memory.reserve(OTHER_ID).unwrap();

    assert_eq!(memory.recall(), vec![remembered(ID, 0)]);
}

fn folder(scratch: &Path) -> PathBuf {
    scratch.join("harbour/sessions")
}

fn remembered(id: &str, position: usize) -> Remembered {
    Remembered {
        session_id: id.to_string(),
        directory: PathBuf::from("/home/acme/api"),
        title: "Fix the flaky login test".to_string(),
        position,
    }
}

fn json(id: &str, position: usize) -> String {
    format!(
        r#"{{"version":1,"sessionId":"{id}","directory":"/home/acme/api","title":"Fix the flaky login test","position":{position}}}"#
    )
}

fn newer(id: &str) -> String {
    format!(
        r#"{{"version":2,"sessionId":"{id}","directory":"/home/acme/api","title":"Fix the flaky login test","position":0}}"#
    )
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn mode(path: &Path) -> u32 {
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn replacing(id: &str, old: &str) -> String {
    format!(
        r#"{{"version":1,"sessionId":"{id}","directory":"/home/acme/api","title":"Fix the flaky login test","position":0,"replaces":"{old}"}}"#
    )
}

fn age(path: &Path, by: Duration) {
    fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(SystemTime::now() - by)
        .unwrap();
}
