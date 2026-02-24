use alloc::string::{String, ToString};
use anyhow::{Result, anyhow};

#[cfg(target_os = "windows")]
use alloc::vec::Vec;
#[cfg(target_os = "windows")]
use windows_sys::Win32::NetworkManagement::NetManagement::{
    LOCALGROUP_MEMBERS_INFO_3, NetLocalGroupAddMembers, NetUserAdd, UF_NORMAL_ACCOUNT, UF_SCRIPT,
    USER_INFO_1, USER_PRIV_USER,
};

#[cfg(target_os = "linux")]
use std::ffi::CString;
#[cfg(target_os = "linux")]
use std::fs::OpenOptions;
#[cfg(target_os = "linux")]
use std::io::Write;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

pub fn create_user(username: String, password: String) -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        create_user_windows(username, password)
    }

    #[cfg(target_os = "linux")]
    {
        create_user_linux(username, password)
    }

    #[cfg(target_os = "macos")]
    {
        Err(anyhow!("Not supported on macOS without system binaries"))
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err(anyhow!("Not supported on this OS"))
    }
}

#[cfg(target_os = "windows")]
fn create_user_windows(username: String, password: String) -> Result<bool> {
    // Convert to wide strings
    let username_wide: Vec<u16> = username.encode_utf16().chain(std::iter::once(0)).collect();
    let password_wide: Vec<u16> = password.encode_utf16().chain(std::iter::once(0)).collect();
    let comment_wide: Vec<u16> = "Created by Eldritch"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let user_info = USER_INFO_1 {
        usri1_name: username_wide.as_ptr() as *mut _,
        usri1_password: password_wide.as_ptr() as *mut _,
        usri1_password_age: 0,
        usri1_priv: USER_PRIV_USER,
        usri1_home_dir: std::ptr::null_mut(),
        usri1_comment: comment_wide.as_ptr() as *mut _,
        usri1_flags: UF_SCRIPT | UF_NORMAL_ACCOUNT,
        usri1_script_path: std::ptr::null_mut(),
    };

    let status = unsafe {
        NetUserAdd(
            std::ptr::null(),
            1,
            &user_info as *const _ as *const u8,
            std::ptr::null_mut(),
        )
    };

    if status != 0 {
        return Err(anyhow!("NetUserAdd failed with status: {}", status));
    }

    // Add to Administrators group
    let group_name = "Administrators"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();

    // Use username_wide again for domainandname
    let group_member_info = LOCALGROUP_MEMBERS_INFO_3 {
        lgrmi3_domainandname: username_wide.as_ptr() as *mut _,
    };

    let status_group = unsafe {
        NetLocalGroupAddMembers(
            std::ptr::null(),
            group_name.as_ptr(),
            3,
            &group_member_info as *const _ as *const u8,
            1,
        )
    };

    if status_group != 0 {
        // Log or ignore failure to add to admin group?
        // We'll consider it a partial success, or maybe we should fail?
        // The prompt says "with administrative/sudo privileges if possible".
        // We can return Ok(true) as user was created.
    }

    Ok(true)
}

#[cfg(target_os = "linux")]
fn create_user_linux(username: String, password: String) -> Result<bool> {
    // 1. Find next UID/GID
    let passwd_content = std::fs::read_to_string("/etc/passwd")?;
    let mut max_uid = 1000;
    for line in passwd_content.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() > 2 {
            if let Ok(uid) = parts[2].parse::<u32>() {
                if uid >= 1000 && uid < 60000 {
                    if uid > max_uid {
                        max_uid = uid;
                    }
                }
            }
        }
    }
    let new_uid = max_uid + 1;
    let new_gid = new_uid; // Create a group with same ID usually

    // 2. Hash password
    let salt_str = generate_salt();
    let hashed_password = crypt_password(&password, &salt_str)?;

    // 3. Append to /etc/passwd
    // username:x:UID:GID:comment:home:shell
    let passwd_entry = format!(
        "{}:x:{}:{}:Eldritch User:/home/{}:/bin/bash\n",
        username, new_uid, new_gid, username
    );
    {
        let mut file = OpenOptions::new().append(true).open("/etc/passwd")?;
        file.write_all(passwd_entry.as_bytes())?;
    }

    // 4. Append to /etc/group (Create new group)
    // groupname:x:GID:
    let group_entry = format!("{}:x:{}:\n", username, new_gid);
    {
        let mut file = OpenOptions::new().append(true).open("/etc/group")?;
        file.write_all(group_entry.as_bytes())?;
    }

    // 5. Append to /etc/shadow
    // username:hash:lastchg:min:max:warn:inactive:expire
    // lastchg is days since epoch
    let days_since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs()
        / 86400;
    let shadow_entry = format!(
        "{}:{}:{}:0:99999:7:::\n",
        username, hashed_password, days_since_epoch
    );
    {
        let mut file = OpenOptions::new().append(true).open("/etc/shadow")?;
        file.write_all(shadow_entry.as_bytes())?;
    }

    // 6. Create home directory
    let home_dir = format!("/home/{}", username);
    if let Err(_) = std::fs::create_dir_all(&home_dir) {
        // Just continue if we can't create home dir? Or fail?
        // Let's assume critical for now but maybe not.
        // Actually, if we can't create home dir, user is still created.
    } else {
        let _ = std::fs::set_permissions(&home_dir, std::fs::Permissions::from_mode(0o755));
        let _ = nix::unistd::chown(
            std::path::Path::new(&home_dir),
            Some(nix::unistd::Uid::from_raw(new_uid)),
            Some(nix::unistd::Gid::from_raw(new_gid)),
        );
    }

    // 7. Add to sudo group
    // Check if 'sudo' or 'wheel' exists
    let group_content = std::fs::read_to_string("/etc/group")?;
    let target_group = if group_content.contains("\nsudo:") {
        "sudo"
    } else if group_content.contains("\nwheel:") {
        "wheel"
    } else {
        ""
    };

    if !target_group.is_empty() {
        let mut new_group_content = String::new();
        for line in group_content.lines() {
            if line.starts_with(&format!("{}:", target_group)) {
                if line.ends_with(':') {
                    new_group_content.push_str(&format!("{}{}", line, username));
                } else {
                    new_group_content.push_str(&format!("{},{}", line, username));
                }
            } else {
                new_group_content.push_str(line);
            }
            new_group_content.push('\n');
        }
        std::fs::write("/etc/group", new_group_content)?;
    }

    Ok(true)
}

#[cfg(target_os = "linux")]
fn generate_salt() -> String {
    use std::io::Read;
    let mut file = match std::fs::File::open("/dev/urandom") {
        Ok(f) => f,
        Err(_) => return "$6$default$".to_string(), // Fallback if urandom fails
    };
    let mut buf = [0u8; 8];
    if file.read_exact(&mut buf).is_err() {
        return "$6$default$".to_string();
    }
    // Simple hex encoding
    let salt: String = buf.iter().map(|b| format!("{:02x}", b)).collect();
    format!("$6${}", salt)
}

#[cfg(target_os = "linux")]
fn crypt_password(password: &str, salt: &str) -> Result<String> {
    use std::ffi::CStr;

    let c_password = CString::new(password)?;
    let c_salt = CString::new(salt)?;

    unsafe extern "C" {
        fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
    }

    unsafe {
        let res = crypt(c_password.as_ptr(), c_salt.as_ptr());
        if res.is_null() {
            return Err(anyhow!("crypt failed"));
        }
        let res_str = CStr::from_ptr(res).to_string_lossy().into_owned();
        Ok(res_str)
    }
}
