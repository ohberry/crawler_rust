use std::time::SystemTime;

use rc4::Rc4;
use rc4::{KeyInit, StreamCipher};

const LIST: [u8; 103] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 10, 11, 12, 13, 14, 15,
];
const CHARACTER: &str = "Dkdpgh4ZKsQB80/Mfvw36XI1R25-WUAlEi7NLboqYTOPuzmFjJnryx9HVGcaStCe=";

fn md52array(md5: &str) -> Vec<u8> {
    let len = md5.len();
    let mut arr: Vec<u8> = Vec::new();
    if len > 32 {
        arr = md5.chars().map(|c| c as u8).collect();
    } else {
        let mut idx = 0;
        while idx < len {
            let c1 = LIST[md5.chars().nth(idx).unwrap() as usize] << 4;
            let c2 = LIST[md5.chars().nth(idx + 1).unwrap() as usize];
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

fn encoding_conversion(arr: &Vec<u8>) -> String {
    let mut y = vec![arr[0]];
    let mut temp = vec![
        arr[10], arr[1], arr[11], arr[2], arr[12], arr[3], arr[13], arr[4], arr[14], arr[5],
        arr[15], arr[6], arr[16], arr[7], arr[17], arr[8], arr[18], arr[9],
    ];
    y.append(&mut temp);
    let character: String = y.iter().map(|&item| item as char).collect();
    character
}

fn encoding_conversion2(a: u8, b: u8, c: String) -> String {
    //将三个参数合并为一个字符串
    let mut arr3 = vec![a, b];
    arr3.append(&mut c.chars().map(|c| c as u8).collect());
    arr3.iter().map(|&item| item as char).collect()
}

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

pub fn generate_xb(url_path: &str) -> String {
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
    let ct: u32 = 536919696;
    let mut arr3: Vec<u8> = vec![];
    let mut arr4: Vec<u8> = vec![];
    let mut new_arr: Vec<u8> = vec![
        64,
        0,
        1,
        8,
        url_path_array[14],
        url_path_array[15],
        arr2[14],
        arr2[15],
        arr1[14],
        arr1[15],
        (timestamp >> 24 & 255) as u8,
        (timestamp >> 16 & 255) as u8,
        (timestamp >> 8 & 255) as u8,
        (timestamp & 255) as u8,
        (ct >> 24 & 255) as u8,
        (ct >> 16 & 255) as u8,
        (ct >> 8 & 255) as u8,
        (ct & 255) as u8,
    ];
    let mut xor_result = new_arr[0];
    for i in 1..new_arr.len() {
        xor_result ^= new_arr[i];
    }
    new_arr.push(xor_result);
    let mut idx = 0;
    while idx < new_arr.len() {
        arr3.push(new_arr[idx]);
        if let Some(&value) = new_arr.get(idx + 1) {
            arr4.push(value);
        }
        idx += 2;
    }
    arr3.append(&mut arr4);
    //将函数结果转为字节数组
    let mut data: Vec<u8> = encoding_conversion(&arr3)
        .chars()
        .map(|c| c as u8)
        .collect();
    let mut rc4 = Rc4::new(b"\xFF".into());
    rc4.apply_keystream(&mut data);
    let garbled_code: String = data.iter().map(|&item| item as char).collect();
    let garbled_code = encoding_conversion2(2, 255, garbled_code);
    let mut idx = 0;
    let mut xb = String::new();
    while idx < garbled_code.chars().count() {
        let temp = calculation(
            garbled_code.chars().nth(idx).unwrap() as i32,
            garbled_code.chars().nth(idx + 1).unwrap() as i32,
            garbled_code.chars().nth(idx + 2).unwrap() as i32,
        );
        xb.push_str(&temp);
        idx += 3;
    }
    xb
}

#[cfg(test)]
mod tests {
    use rc4::Rc4;
    use rc4::{KeyInit, StreamCipher};

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

    #[test]
    fn test_xb() {
        let result =generate_xb("aid=6383&sec_user_id=MS4wLjABAAAA5fOskZLfiDB9wvPP0LHJB_tDvwTnzKL2K3Cj1C-81YczaAhHUvFr-7BnpZ-yOiX6&count=20&max_cursor=0&cookie_enabled=true&platform=PC&downlink=10");
        println!("{}", result);
    }
}
