use std::env;
use std::fs::File;
#[cfg(test)]

use xattr::FileExt;

#[test] #[ignore]
fn test_create_repo() {

}

#[test]
fn test_write_xattr() -> Result<(), &'static str> {
    let dir = env::temp_dir();
    let fp = dir.join("com.remote-text-.server.test.dependency-tests.test-write-xattr");
    let f = File::create(&fp).map_err(|_| "Unable to create file")?;

    f.set_xattr("xattr_name", "hello".as_ref()).map_err(|_| "Unable to set xattr on file")?;

    xattr::set(&fp, "xattr_name_2", "goodbye".as_ref()).map_err(|_| "Unable to set xattr from filepath")
}

#[test]
fn test_read_xattr() -> Result<(), &'static str> {
    let dir = env::temp_dir();
    let fp = dir.join("com.remote-text-.server.test.dependency-tests.test-read-xattr");
    let f = File::create(&fp).map_err(|_| "Unable to create file")?;

    f.set_xattr("xattr_name", "hello".as_ref()).map_err(|_| "Unable to set xattr on file")?;

    xattr::set(&fp, "xattr_name_2", "goodbye".as_ref()).map_err(|_| "Unable to set xattr from filepath")?;


    let x = f.get_xattr("xattr_name_2")
        .map_err(|_| "I/O error reading xattr 2")?
        .ok_or("Unable to get xattr 2 from file")?;
    assert_eq!("goodbye".to_string(),
               String::from_utf8(x)
                   .map_err(|_| "Unable to convert xattr2 to UTF-8")?);

    let x2 = xattr::get(fp, "xattr_name")
        .map_err(|_| "I/O error reading xattr 1")?
        .ok_or("Unable to get xattr from filepath")?;
    assert_eq!("hello".to_string(),
               String::from_utf8(x2)
                   .map_err(|_| "Unable to convert xattr to UTF-8")?);

    Ok(())
}

#[test]
fn test_xattr_is_supported_platform() {
    assert!(xattr::SUPPORTED_PLATFORM);
}