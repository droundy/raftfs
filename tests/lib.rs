use std::io::{Write, Read};

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
        TempDir(std::path::PathBuf::from(&p), s.unwrap())
    }
    fn path(&self, p: &str) -> std::path::PathBuf {
        self.0.join(p)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        self.1.kill().ok();
        std::fs::remove_dir_all(&self.0).ok(); // ignore errors that might happen on windows
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

#[test]
fn nothing() {
    TempDir::new(&format!("tests/test-{}", line!()));
}

#[test]
fn read_empty() {
    let t = TempDir::new(&format!("tests/test-{}", line!()));
    for entry in std::fs::read_dir(t.path("mnt")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        println!("entry: {:?}", &path);
        assert!(false);
    }
}
