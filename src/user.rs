pub fn get_uid_from_username(username: &str) -> u32 {
    let c_username = std::ffi::CString::new(username).unwrap();

    unsafe {
        let passwd_entry = libc::getpwnam(c_username.as_ptr());

        if !passwd_entry.is_null() {
            (*passwd_entry).pw_uid
        } else {
            panic!("unable to get uid from username, use uid to skip this step");
        }
    }
}
