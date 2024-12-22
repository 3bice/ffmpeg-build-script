use std::{
    env::{self},
    ffi::OsStr,
    io::Write,
    path::PathBuf,
    process::{exit, Command},
    str::FromStr,
};

fn command<T: AsRef<OsStr>>(cmd: &str, args: Vec<T>) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new(cmd).args(args).output();

    let _out = match output {
        Ok(output) => {
            if output.status.success() {
                // println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                eprintln!("Command failed with status: {}", output.status);
                eprintln!("Error Output:\n{}", String::from_utf8_lossy(&output.stderr));
                return Err(String::from_utf8_lossy(&output.stderr).into());
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            return Err(e.into());
        }
    };
    Ok(_out)
}

fn modify(
    current_dir: PathBuf,
    file_path: PathBuf,
    have_change: &mut bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let out = match command("otool", vec!["-L", &file_path.to_str().unwrap()]) {
        Ok(out) => out,
        Err(e) => return Err(e),
    };

    for line in out.lines() {
        if line.trim().starts_with("@rpath") {
            let lib_name = line.split(" ").next().unwrap().split("/").last().unwrap();
            if !current_dir.join(lib_name).exists() {
                // let out = match command("find", vec!["/", "-name", lib_name, "-maxdepth", "5"]) {
                //     Ok(out) => out,
                //     Err(e) => return Err(e),
                // };
                // eprintln!("find libs = {:?}", out);
                // for line in out.lines() {
                //     if line.trim().starts_with("find:") {
                //         continue;
                //     }
                //     eprintln!("start copy line = {:?}", line);
                //     std::fs::copy(line.trim(), current_dir.join(lib_name)).unwrap();
                //     break;
                // }
                *have_change = true;
                println!(
                    "path not exists: {}",
                    current_dir.join(lib_name).to_str().unwrap()
                );
                let sys_lib_path = PathBuf::from_str("/usr/local/lib")?.join(lib_name);
                if sys_lib_path.exists() {
                    println!("start copy {}.", lib_name);
                    std::fs::copy(sys_lib_path, current_dir.join(lib_name)).unwrap();
                } else {
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(current_dir.parent().unwrap().join("result.txt"))
                        .unwrap();
                    writeln!(file, "{} -> {}", file_path.to_str().unwrap(), lib_name).unwrap();
                }
            }
            continue;
        }
        if !line.trim().starts_with("/usr/local/opt")
            && !line.trim().starts_with("/opt/homebrew")
            && !line.trim().starts_with("/usr/local/Cellar")
        {
            continue;
        }
        eprintln!("--- line: {}", line);

        let link_to_lib_path = line.trim().split(" ").next().unwrap();
        eprintln!("link_to_lib_path = {:?}", link_to_lib_path);
        let link_to_lib_path_buf = PathBuf::from(link_to_lib_path);
        let file_name = link_to_lib_path_buf.file_name().unwrap().to_str().unwrap();
        let copy_to_path = file_path.parent().unwrap().join(file_name);
        println!("copy to: {:?}", copy_to_path);

        // file not in lib folder
        if !copy_to_path.exists() {
            println!("file not exists: {:?} start copy...", copy_to_path);
            std::fs::copy(link_to_lib_path, copy_to_path)?;
        } else {
            continue;
        }
        *have_change = true;
        let origin_lib_file_name = file_path.file_name().unwrap().to_str().unwrap();
        eprintln!("origin_lib_file_name = {:?}", origin_lib_file_name);
        if line.contains(origin_lib_file_name) {
            let _out = match command(
                "install_name_tool",
                vec![
                    "-id",
                    &format!("@rpath/{}", origin_lib_file_name),
                    origin_lib_file_name,
                ],
            ) {
                Ok(out) => out,
                Err(e) => return Err(e),
            };
            continue;
        }

        let _ = command(
            "install_name_tool",
            vec![
                "-change",
                link_to_lib_path,
                &format!("@loader_path/{}", file_name),
                file_path.to_str().unwrap(),
            ],
        );
    }

    Ok(())
}

fn main() {
    let mut i = 1;
    loop {
        println!("=================> round {}", i);
        i = i + 1;
        let mut have_change = false;
        let current_dir = match env::current_dir() {
            Ok(current_dir) => current_dir,
            Err(e) => {
                eprintln!("Failed to get current directory: {}", e);
                exit(1);
            }
        };

        println!("current dir: {:?}", current_dir);
        let dir_content = std::fs::read_dir(current_dir.clone()).unwrap();

        for entry in dir_content {
            let entry = entry.unwrap();
            let file_path = entry.path();

            println!("=== file: {:?}", file_path);

            match modify(current_dir.clone(), file_path.clone(), &mut have_change) {
                Ok(_) => continue,
                Err(e) => println!("x file_path:{}, error: {}", file_path.to_str().unwrap(), e),
            }

            if !have_change {
                break;
            }
        }
    }
}
