use std::env;
use std::fs::File;
#[cfg(test)]

use xattr::FileExt;

#[test] #[ignore]
fn test_create_repo() {

}

#[test]
fn test_write_xattr() -> Result<(), String> {
    let dir = env::temp_dir();
    let fp = dir.join("com.remote-text.server.test.dependency-tests.test-write-xattr");
    let f = File::create(&fp).map_err(|_| "Unable to create file")?;

    f.set_xattr("user.xattr_name", "h".as_ref()).map_err(|e| e.to_string())?;

    xattr::set(&fp, "user.xattr_name_2", "goodbye".as_ref()).map_err(|e| e.to_string())
}

#[test]
fn test_read_xattr() -> Result<(), String> {
    let dir = env::temp_dir();
    let fp = dir.join("com.remote-text.server.test.dependency-tests.test-read-xattr");
    let f = File::create(&fp).map_err(|_| "Unable to create file")?;

    f.set_xattr("user.xattr_name", "hello".as_ref()).map_err(|e| e.to_string())?;

    xattr::set(&fp, "user.xattr_name_2", "goodbye".as_ref()).map_err(|e| e.to_string())?;


    let x = f.get_xattr("user.xattr_name_2")
        .map_err(|e| e.to_string())?
        .ok_or("xattr 2 returned nil (from file)")?;
    assert_eq!("goodbye".to_string(),
               String::from_utf8(x)
                   .map_err(|_| "Unable to convert xattr2 to UTF-8")?);

    let x2 = xattr::get(fp, "user.xattr_name")
        .map_err(|e| e.to_string())?
        .ok_or("xattr 1 returned nil (from filepath)")?;
    assert_eq!("hello".to_string(),
               String::from_utf8(x2)
                   .map_err(|_| "Unable to convert xattr to UTF-8")?);

    Ok(())
}

#[test]
fn test_xattr_is_supported_platform() {
    assert!(xattr::SUPPORTED_PLATFORM);
}