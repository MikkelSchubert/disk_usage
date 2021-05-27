extern crate libc;
extern crate nix;


pub fn get_username(uid: u32) -> Result<String, String> {
    let initial_size = ::std::cmp::max(libc::_SC_GETPW_R_SIZE_MAX, 70);
    let mut buff = vec![0; initial_size as usize];

    unsafe {
        let mut passwd: libc::passwd = ::std::mem::zeroed();
        let mut result: *mut libc::passwd = ::std::ptr::null_mut();

        loop {
            let err = libc::getpwuid_r(uid, &mut passwd,
                                       buff.as_mut_ptr(),
                                       buff.len(),
                                       &mut result);

            if err == nix::Errno::ERANGE as i32 {
                let len = buff.len();
                buff.resize(len * 2, 0);
            } else if err == 0 && result == &mut passwd && !passwd.pw_name.is_null() {
                let cstr = ::std::ffi::CStr::from_ptr(passwd.pw_name);

                return if let Ok(s) = cstr.to_str() {
                    Ok(s.into())
                } else {
                    Err(format!("Error converting username to str for {}", uid))
                }
            } else {
                return Err(format!("Error retrieving username for {}: {}", uid, err));
            }
        }
    }
}
