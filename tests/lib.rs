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
    (fn $testname:ident($t:ident) $body:expr) => {
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
                // Remove temporary directoyr, but ignore errors that
                // might happen on windows.
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
