use std::{env, path::Path};

fn is_relative_path(path_str: &String) -> bool {
    let path = Path::new(&path_str);
    path.is_relative()
}

pub fn normalize_path(path_str: String) -> String {
    if is_relative_path(&path_str) {
        let pwd = env::current_dir().expect("unable to get cwd");
        let config_path = Path::new(pwd.as_path()).join(path_str);

        config_path.into_os_string().into_string().unwrap()
    } else {
        Path::new(&path_str)
            .to_path_buf()
            .into_os_string()
            .into_string()
            .unwrap()
    }
}
