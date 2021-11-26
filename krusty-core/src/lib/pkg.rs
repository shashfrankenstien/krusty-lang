use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use dirs;

use crate::syntax::evaluator::NameSpace;
use crate::lib::errors::{Error, KrustyErrorType};

pub const INSTALL_FOLDER: &'static str = ".krusty";
pub const INSTALL_SUBFOLDER: &'static str = "pkg";
pub const LANGUAGE_EXT: &'static str = "krt";
pub const DIR_PKG_INITIALIZER: &'static str = "__pkg__.krt";

const DYLIB_INIT_COMMENT: &'static str = r#"# This file was created by Krusty's --install option
# - allows for easy importing of native dylib package

"#;


fn normalize_import_path(p: &mut PathBuf) {
    if p.is_dir() {
        // DIR_PKG_INITIALIZER file is imported if import() is used on the parent directory
        // Might be useful to write native modules
        p.push(DIR_PKG_INITIALIZER);
    } else if !p.ends_with(LANGUAGE_EXT) {
        p.set_extension(LANGUAGE_EXT);
    }
}

pub fn search_for_module(ns: &NameSpace, name: &String) -> Result<PathBuf, KrustyErrorType> {
    // Checks for non native modules or files in different locations
    // 1. Check current working directory
    let mut cwd = env::current_dir().unwrap_or(PathBuf::from("."));
    cwd.push(name);
    normalize_import_path(&mut cwd);

    if cwd.is_file() {
        print_verbose!("current dir");
        return Ok(cwd)
    }

    // 2. Check relative path to calling module / namespace
    let mut relative = ns.get_relative_path(name);
    normalize_import_path(&mut relative);

    if relative.is_file() {
        print_verbose!("relative dir");
        return Ok(relative)
    }

    // 3. Check pkg installation directory
    let mut pkg_path = dirs::home_dir().ok_or("HOME dir not found")?;
    pkg_path.push(INSTALL_FOLDER);
    pkg_path.push(INSTALL_SUBFOLDER);
    pkg_path.push(name);
    normalize_import_path(&mut pkg_path);

    if pkg_path.is_file() {
        print_verbose!("pkg dir");
        return Ok(pkg_path)
    }

    import_error!(format!("'{}' Not found", name))
}



pub fn to_native_dylib_name(p: &mut PathBuf) -> Result<(), KrustyErrorType> {
    let mut fname = p.file_name()
        .ok_or("filename not valid")?
        .to_str()
        .ok_or("filename not valid")?
        .to_owned();

    #[cfg(target_os = "windows")]
    {
        if !fname.ends_with(".dll") {
            fname = fname + ".dll"
        }
        p.set_file_name(fname);
    }
    #[cfg(target_os = "macos")]
    {
        if !fname.starts_with("lib") {
            fname = "lib".to_owned() + &fname;
        }
        if !fname.ends_with(".dylib") {
            fname = fname + ".dylib"
        }
        p.set_file_name(fname);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if !fname.starts_with("lib") {
            fname = "lib".to_owned() + &fname;
        }
        if !fname.ends_with(".so") {
            fname = fname + ".so"
        }
        p.set_file_name(fname);
    }

    Ok(())
}





pub fn copy_dir<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), KrustyErrorType> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }
    Ok(())
}


pub fn install_pkg(path_str: &String) -> Result<(), KrustyErrorType> {
    // installs a package directory, language file or native dylib
    // 1. test for package directory (must contain DIR_PKG_INITIALIZER file)
    let pkg_path = PathBuf::from(path_str);

    let mut dst_path = dirs::home_dir().ok_or("HOME dir not found")?;
    dst_path.push(INSTALL_FOLDER);
    dst_path.push(INSTALL_SUBFOLDER);

    if pkg_path.is_dir() {
        let mut init_file = PathBuf::from(path_str);
        init_file.push(DIR_PKG_INITIALIZER);
        if !init_file.is_file() {
            generic_error!(format!("Package does not contain '{}'", DIR_PKG_INITIALIZER))
        }

        let dirname = pkg_path.file_name()
            .ok_or("filename not valid")?
            .to_str()
            .ok_or("filename not valid")?
            .to_owned();

        dst_path.push(dirname);
        copy_dir(&pkg_path, &dst_path)?;

    } else {
        let ext = pkg_path.extension()
            .ok_or("ext not valid")?
            .to_str()
            .ok_or("ext not valid")?
            .to_owned();

        if ext == LANGUAGE_EXT {
            let filename = pkg_path.file_name()
                .ok_or("filename not valid")?
                .to_str()
                .ok_or("filename not valid")?
                .to_owned();
            dst_path.push(&filename);
            println!("copy: {:?}", filename);
            fs::copy(&pkg_path, &dst_path)?;

        } else {
            let mut fstem = pkg_path.file_stem()
                .ok_or("filename not valid")?
                .to_str()
                .ok_or("filename not valid")?
                .to_owned();

            if fstem.starts_with("lib") {
                fstem = fstem[3..].to_string();
            }

            dst_path.push(fstem.clone());
            if fs::metadata(&dst_path).is_err() {
                println!(" mkdir: {:?}", dst_path);
                fs::create_dir_all(&dst_path)?;
            }

            dst_path.push(DIR_PKG_INITIALIZER);
            let mut init_file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&dst_path)?;

            init_file.write_all(DYLIB_INIT_COMMENT.as_bytes())?;
            init_file.write_all(format!("spill(import_native(\"{}\"))\n", &fstem).as_bytes())?;
            println!("  create: {:?}", DIR_PKG_INITIALIZER);

            // generic_error!(format!("{:?} not implemented", pkg_path));
            let fname = pkg_path.file_name()
                .ok_or("filename not valid")?
                .to_str()
                .ok_or("filename not valid")?
                .to_owned();

            dst_path.set_file_name(&fname);
            fs::copy(&pkg_path, &dst_path)?;
            println!("  copy: {:?}", fname);

        }
    }

    Ok(())
}
