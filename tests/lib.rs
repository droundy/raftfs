use std::io::{Write, Read};

#[derive(Debug)]
struct TempDir(std::path::PathBuf, std::process::Child);
impl TempDir {
    fn new<P: AsRef<std::path::Path>> (p: P) -> TempDir {
        let here = std::env::current_dir().unwrap();
        let p = here.join(p);
        println!("remove test repository");
        std::fs::remove_dir_all(&p).ok();
        println!("create {:?}", &p);
        assert!(std::fs::create_dir_all(&p.join("data")).is_ok());
        assert!(std::fs::create_dir_all(&p.join("mnt")).is_ok());
        println!("current_dir = {:?}", &p);
        let e = location_of_executables().join("raftfs");
        println!("executable = {:?}", &e);
        // Now run raftfs to mount us
        let s = std::process::Command::new(e)
            .args(&["data", "mnt"])
            .current_dir(&p).spawn();
        if !s.is_ok() {
            println!("Bad news: {:?}", s);
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
        TempDir(std::path::PathBuf::from(&p), s.unwrap())
    }
    fn path(&self, p: &str) -> std::path::PathBuf {
        self.0.join(p)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        self.1.kill().ok();
    }
}

fn location_of_executables() -> std::path::PathBuf {
    // The key here is that this test executable is located in almost
    // the same place as the built `fac` is located.
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // chop off exe name
    path.pop(); // chop off "deps"
    path
}

macro_rules! test_case {
    (fn $testname:ident($t:ident) $body:block) => {
        mod $testname {
            use super::*;
            #[test]
            fn $testname() {
                let path = std::path::PathBuf::from(
                    format!("tmp/{}", module_path!()));
                {
                    let $t = TempDir::new(&path);
                    $body;
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
                // Remove temporary directory (we did not panic!), but
                // ignore errors that might happen on windows.
                std::fs::remove_dir_all(&path).ok();
            }
        }
    }
}

test_case!{
    fn nothing(t) {
        println!("Testing: {:?}", t);
    }
}

test_case!{
    fn read_empty_directory(t) {
        for entry in std::fs::read_dir(t.path("mnt")).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            println!("entry: {:?}", &path);
            assert!(false);
        }
    }
}

test_case!{
    fn file_write_read(t) {
        let contents = b"hello\n";
        {
            let mut f = std::fs::File::create(t.path("mnt/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        {
            let mut f = std::fs::File::open(t.path("mnt/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file actually got stored in data");
            let mut f = std::fs::File::open(t.path("data/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
    }
}

test_case!{
    fn file_write_read_snapshot(t) {
        let contents = b"hello\n";
        {
            let mut f = std::fs::File::create(t.path("mnt/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        {
            let mut f = std::fs::File::open(t.path("mnt/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");
        {
            println!("verify that the file actually got stored in data");
            let mut f = std::fs::File::open(t.path("data/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file can be read from the snapshot.");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
    }
}

test_case!{
    fn file_rename(t) {
        let contents = b"hello\n";
        {
            let mut f = std::fs::File::create(t.path("mnt/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        assert!(std::fs::File::open(t.path("mnt/testfile")).is_ok());
        assert!(std::fs::File::open(t.path("mnt/newname")).is_err());
        std::fs::rename(t.path("mnt/testfile"), t.path("mnt/newname")).unwrap();
        assert!(std::fs::File::open(t.path("mnt/testfile")).is_err());
        assert!(std::fs::File::open(t.path("data/testfile")).is_err());
        assert!(std::fs::File::open(t.path("mnt/newname")).is_ok());
        assert!(std::fs::File::open(t.path("data/newname")).is_ok());
        {
            let mut f = std::fs::File::open(t.path("mnt/newname")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file actually got stored in data");
            let mut f = std::fs::File::open(t.path("data/newname")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
    }
}

test_case!{
    fn file_rename_in_snapshot(t) {
        let contents = b"hello\n";
        {
            let mut f = std::fs::File::create(t.path("mnt/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        {
            let mut f = std::fs::File::open(t.path("mnt/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");
        {
            println!("verify that the file actually got stored in data");
            let mut f = std::fs::File::open(t.path("data/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file can be read from the snapshot.");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        let e = std::fs::rename(t.path("mnt/.snapshots/snap/testfile"), t.path("mnt/.snapshots/snap/newname"));
        println!("rename gives: {:?}", &e);
        assert!(e.is_err());
    }
}

test_case!{
    fn file_readdir_of_snapshot(t) {
        let contents = b"hello\n";
        {
            let mut f = std::fs::File::create(t.path("mnt/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        {
            let mut f = std::fs::File::open(t.path("mnt/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");
        {
            println!("verify that the file actually got stored in data");
            let mut f = std::fs::File::open(t.path("data/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file can be read from the snapshot.");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        assert!(!t.path("data/.snapshots/snap/testfile").exists());
        let mut visited = std::collections::HashSet::new();
        for entry in std::fs::read_dir(t.path("mnt/.snapshots/snap")).unwrap() {
            let entry = entry.unwrap();
            let path = entry.file_name();
            println!("path: {:?}", path);
            visited.insert(std::path::PathBuf::from(path));
        }
        assert!(visited.contains(std::path::Path::new("testfile")));
    }
}

test_case!{
    fn file_rename_snapshot(t) {
        let contents = b"hello\n";
        {
            let mut f = std::fs::File::create(t.path("mnt/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        {
            let mut f = std::fs::File::open(t.path("mnt/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");
        {
            println!("verify that the file actually got stored in data");
            let mut f = std::fs::File::open(t.path("data/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file can be read from the snapshot.");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        std::fs::rename(t.path("mnt/testfile"), t.path("mnt/newname")).unwrap();
        assert!(t.path("mnt/.snapshots/snap/testfile").exists());
        assert!(!t.path("mnt/.snapshots/snap/newname").exists());
        assert!(std::fs::File::open(t.path("mnt/testfile")).is_err());
        assert!(std::fs::File::open(t.path("data/testfile")).is_err());
        assert!(std::fs::File::open(t.path("mnt/.snapshots/snap/testfile")).is_ok());
        assert!(std::fs::File::open(t.path("mnt/newname")).is_ok());
        assert!(std::fs::File::open(t.path("data/newname")).is_ok());
        assert!(std::fs::File::open(t.path("mnt/.snapshots/snap/newname")).is_err());
        {
            let mut f = std::fs::File::open(t.path("mnt/newname")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file actually got stored in data");
            let mut f = std::fs::File::open(t.path("data/newname")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
    }
}

test_case!{
    fn file_directory_in_snapshot(t) {
        std::fs::create_dir(t.path("mnt/testdir")).unwrap();
        assert!(t.path("mnt/testdir").is_dir());

        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");

        assert!(t.path("mnt/.snapshots/snap/testdir").is_dir());
    }
}

test_case!{
    fn rmdir_in_snapshot(t) {
        std::fs::create_dir(t.path("mnt/testdir")).unwrap();
        assert!(t.path("mnt/testdir").is_dir());

        assert!(t.path("data/testdir").is_dir());
        assert!(!t.path("mnt/.snapshots/snap/testdir").is_dir());

        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");

        assert!(t.path("mnt/.snapshots/snap/testdir").is_dir());

        assert!(t.path("data/testdir").is_dir());
        assert!(t.path("mnt/.snapshots/snap/testdir").is_dir());
        assert!(!t.path("data/.snapshots/snap/testdir").is_dir());

        std::fs::remove_dir(t.path("mnt/testdir")).unwrap();
        assert!(!t.path("mnt/testdir").is_dir());
        assert!(!t.path("data/testdir").is_dir());
        assert!(t.path("mnt/.snapshots/snap/testdir").is_dir());
    }
}

test_case!{
    fn file_mkdir_after_snapshot(t) {
        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");

        assert!(!t.path("mnt/testdir").is_dir());
        assert!(!t.path("data/testdir").is_dir());
        assert!(!t.path("mnt/.snapshots/snap/testdir").is_dir());
        assert!(!t.path("data/.snapshots/snap/testdir").is_dir());

        std::fs::create_dir(t.path("mnt/testdir")).unwrap();
        assert!(t.path("mnt/testdir").is_dir());
        assert!(t.path("data/testdir").is_dir());
        assert!(!t.path("data/.snapshots/snap/testdir").is_dir());
        assert!(!t.path("mnt/.snapshots/snap/testdir").is_dir());
    }
}

test_case!{
    fn mkdir_in_snapshot(t) {
        std::fs::create_dir_all(t.path("mnt/subdir")).unwrap();
        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");

        assert!(t.path("mnt/subdir").is_dir());
        assert!(t.path("mnt/.snapshots/snap/subdir").is_dir());

        assert!(std::fs::create_dir(t.path("mnt/.snapshots/snap/testdir")).is_err());
        assert!(std::fs::create_dir(t.path("mnt/.snapshots/snap/subdir/testdir")).is_err());
    }
}

test_case!{
    fn unlink_after_snapshot(t) {
        let contents = b"hello\n";
        {
            let mut f = std::fs::File::create(t.path("mnt/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        std::fs::create_dir_all(t.path("mnt/subdir")).unwrap();
        {
            let mut f = std::fs::File::create(t.path("mnt/subdir/testfile")).unwrap();
            f.write(contents).unwrap();
        }
        println!("creating .snapshots");
        std::fs::create_dir_all(t.path("mnt/.snapshots/snap")).unwrap();
        println!("done creating .snapshots/snap");
        {
            println!("verify that the file is in snapshot");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the file is present");
            let mut f = std::fs::File::open(t.path("mnt/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }

        assert!(t.path("mnt/subdir").is_dir());
        assert!(t.path("mnt/subdir/testfile").is_file());
        assert!(t.path("mnt/.snapshots/snap/subdir").is_dir());
        assert!(std::fs::remove_file(t.path("mnt/.snapshots/snap/testfile")).is_err());
        std::fs::remove_file(t.path("mnt/testfile")).unwrap();
        assert!(!t.path("mnt/testfiler").exists());
        assert!(t.path("mnt/.snapshots/snap/testfile").exists());
        {
            println!("verify that the file is correct in snapshot");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
        {
            println!("verify that the subdir file is correct in snapshot");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/subdir/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }

        assert!(t.path("data/subdir").is_dir());
        assert!(t.path("data/subdir/testfile").is_file());
        assert!(!t.path("data/.snapshots/snap/subdir/testfile").is_file());
        assert!(t.path("mnt/subdir").is_dir());
        assert!(t.path("mnt/subdir/testfile").is_file());

        //std::fs::remove_dir_all(t.path("mnt/.snapshots/snap/subdir")).is_err();
        assert!(t.path("data/subdir").is_dir());
        assert!(t.path("data/subdir/testfile").is_file());
        assert!(!t.path("data/.snapshots/snap/subdir/testfile").is_file());
        assert!(t.path("mnt/subdir").is_dir());
        assert!(t.path("mnt/subdir/testfile").is_file());
        println!("remove file from subdir");
        //std::fs::remove_file(t.path("mnt/subdir/testfile")).unwrap();
        std::fs::remove_dir_all(t.path("mnt/subdir")).unwrap();
        assert!(!t.path("mnt/subdir").is_dir());
        assert!(!t.path("mnt/subdir/testfile").is_file());
        assert!(t.path("mnt/.snapshots/snap/subdir").is_dir());
        assert!(t.path("mnt/.snapshots/snap/subdir/testfile").is_file());

        println!("we should have written subdir to the snap directory");
        assert!(t.path("data/.snapshots/snap/subdir").is_dir());
        assert!(t.path("data/.snapshots/snap/subdir/testfile").is_file());
        {
            println!("verify that the subdir file is still correct in snapshot");
            let mut f = std::fs::File::open(t.path("mnt/.snapshots/snap/subdir/testfile")).unwrap();
            let mut actual_contents = Vec::new();
            f.read_to_end(&mut actual_contents).unwrap();
            assert_eq!(std::str::from_utf8(&actual_contents),
                       std::str::from_utf8(contents));
        }
    }
}

