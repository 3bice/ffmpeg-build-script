use std::{
    env::{self},
    io::Write,
    path::PathBuf,
    process::{exit, Command},
};

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

            println!("file_path: {:?}", file_path);
            let output = Command::new("otool").arg("-L").arg(&file_path).output();

            let out = match output {
                Ok(output) => {
                    if output.status.success() {
                        // println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                        String::from_utf8_lossy(&output.stdout).to_string()
                    } else {
                        eprintln!("Command failed with status: {}", output.status);
                        eprintln!("Error Output:\n{}", String::from_utf8_lossy(&output.stderr));
                        exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute command: {}", e);
                    exit(1);
                }
            };

            for line in out.lines() {
                if !line.trim().starts_with("/usr/local/opt")
                    && !line.trim().starts_with("/opt/homebrew")
                    && !line.trim().starts_with("/usr/local/Cellar")
                {
                    continue;
                }
                eprintln!("--- {}", line);
                have_change = true;
                let link_to_lib_path = line.trim().split(" ").next().unwrap();
                eprintln!("link_to_lib_path = {:?}", link_to_lib_path);
                let link_to_lib_path_buf = PathBuf::from(link_to_lib_path);
                let file_name = link_to_lib_path_buf.file_name().unwrap().to_str().unwrap();
                println!("dest file: {:?}", file_path.join(file_name));
                if !file_path.parent().unwrap().join(file_name).exists() {
                    if !link_to_lib_path_buf.exists() {
                        let mut file = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(current_dir.parent().unwrap().join("result.txt"))
                            .unwrap();
                        writeln!(file, "{}", link_to_lib_path).unwrap();
                        continue;
                    }
                    std::fs::copy(
                        link_to_lib_path,
                        file_path.parent().unwrap().join(file_name),
                    )
                    .unwrap();
                }

                let origin_lib_file_name = file_path.file_name().unwrap().to_str().unwrap();
                if line.contains(origin_lib_file_name) {
                    let output = Command::new("install_name_tool")
                        .arg("-id")
                        .arg(format!("@rpath/{}", origin_lib_file_name)) // @executable_path @loader_path
                        .arg(origin_lib_file_name)
                        .output();

                    let _out = match output {
                        Ok(output) => {
                            if output.status.success() {
                                // println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                                String::from_utf8_lossy(&output.stdout).to_string()
                            } else {
                                eprintln!("Command failed with status: {}", output.status);
                                eprintln!(
                                    "Error Output:\n{}",
                                    String::from_utf8_lossy(&output.stderr)
                                );
                                exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to execute command: {}", e);
                            exit(1);
                        }
                    };
                    continue;
                }

                let output = Command::new("install_name_tool")
                    .arg("-change")
                    .arg(link_to_lib_path)
                    .arg(format!("@loader_path/{}", file_name)) // @executable_path @loader_path
                    .arg(file_path.to_str().unwrap())
                    .output();

                let _out = match output {
                    Ok(output) => {
                        if output.status.success() {
                            // println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                            String::from_utf8_lossy(&output.stdout).to_string()
                        } else {
                            eprintln!("Command failed with status: {}", output.status);
                            eprintln!("Error Output:\n{}", String::from_utf8_lossy(&output.stderr));
                            exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to execute command: {}", e);
                        exit(1);
                    }
                };
            }
        }

        if !have_change {
            break;
        }
    }
}
