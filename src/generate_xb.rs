use std::{result, time::SystemTime};

const CHARACTER: &str = "Dkdpgh4ZKsQB80/Mfvw36XI1R25-WUAlEi7NLboqYTOPuzmFjJnryx9HVGcaStCe=";

fn md52array(md5: &str) -> Vec<u8> {
    let len = md5.len();
    let mut arr: Vec<u8> = Vec::new();
    if len > 32 {
        arr = md5.chars().map(|c| c as u8).collect();
    } else {
        let mut idx = 0;
        while idx < len {
            let c1 = (md5.chars().nth(idx).unwrap() as u8) << 4;
            let c2 = md5.chars().nth(idx + 1).unwrap() as u8;
            arr.push(c1 | c2);
            idx += 2;
        }
    }
    arr
}

fn md5_encrypt(url_path: &str) -> Vec<u8> {
    let m1 = format!("{:x}", md5::compute(url_path));
    let m2 = md52array(m1.as_str());
    let m3 = format!("{:x}", md5::compute(m2));
    md52array(m3.as_str())
}

fn encoding_conversion() {}

fn calculation(a1: i32, a2: i32, a3: i32) -> String {
    let x1 = (a1 & 255) << 16;
    let x2 = (a2 & 255) << 8;
    let x3 = x1 | x2 | a3;
    let indices = [
        (x3 & 16515072) >> 18,
        (x3 & 258048) >> 12,
        (x3 & 4032) >> 6,
        x3 & 63,
    ];
    indices.iter().fold(String::new(), |mut result, &index| {
        result.push(CHARACTER.chars().nth(index as usize).unwrap());
        result
    })
}

fn generate_xb(url_path:&str) {
    let arr1 = md52array("d88201c9344707acde7261b158656c0e");
    let arr2 = md52array(
        format!(
            "{:x}",
            md5::compute(md52array("d41d8cd98f00b204e9800998ecf8427e"))
        )
        .as_str(),
    );
    let url_path_array = md5_encrypt(url_path);
    let timestamp = SystemTime::now()
    .duration_since(SystemTime::UNIX_EPOCH)
    .unwrap()
    .as_secs();
    let ct = 536919696;
    let arr3:Vec<i32> = vec![];
    let arr4:Vec<i32> =vec![];
}

#[cfg(test)]
mod tests {
    use rc4::{consts::*, KeyInit, StreamCipher};
    use rc4::{Key, Rc4};

    use super::*;

    #[test]
    fn test_md52array() {
        let md5 = "e10adc3949ba59abbe56e057f20f883e";
        let result = md52array(md5);
        println!("{:?}", result);
    }

    #[test]
    fn test_md5() {
        md5_encrypt("123456");
    }

    #[test]
    fn test_rc4() {
        let mut rc4 = Rc4::new(b"Key".into());
        let mut data = b"Plaintext".to_vec();
        rc4.apply_keystream(&mut data);
        // assert_eq!(data, [0xBB, 0xF3, 0x16, 0xE8, 0xD9, 0x40, 0xAF, 0x0A, 0xD3]);
        println!("{:?}", data);
    }
}
