use std::process::{Command, Stdio};

pub fn get_uid_by_name(name: String) -> u32 {
    let out = Command::new("id")
        .args(["-u", name.as_str()])
        .stderr(Stdio::null())
        .output()
        .expect("looks like you don't have an \"id\" utility, use uid instead of username");

    match out.status.code() {
        Some(0) => (),
        Some(code) => panic!("unable to get uid for user \"{}\", status code \"{}\"", name, code),
        None => panic!("unable to get uid for user \"{}\", no status code", name),
    }

    let stdout = match std::str::from_utf8(&out.stdout) {
        Ok(str) => {
            String::from(str)
        },
        Err(_) => {
            panic!("unable to get uid, non utf-8 output");
        }
    };

    stdout.trim().parse::<u32>().expect("unable to get uid, non numeric output")
}