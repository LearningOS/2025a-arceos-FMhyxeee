#![no_std]
#![no_main]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

extern crate alloc;
use alloc::string::String;
use hashbrown::HashMap;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    println!("Running memory tests...");
    test_hashmap();
    println!("Memory tests run OK!");
}

fn test_hashmap() {
    const N: u32 = 50_000;
    let mut m = HashMap::new();
    for value in 0..N {
        // Use a simpler key generation approach
        let mut key = String::new();
        key.push_str("key_");
        // Simple integer to string conversion
        if value == 0 {
            key.push('0');
        } else {
            let mut temp = value;
            let mut digits = [0u8; 10];
            let mut len = 0;
            while temp > 0 {
                digits[len] = (temp % 10) as u8 + b'0';
                temp /= 10;
                len += 1;
            }
            for i in (0..len).rev() {
                key.push(digits[i] as char);
            }
        }
        m.insert(key, value);
    }

    // Test a subset to verify correctness
    for value in 0..1000.min(N) {
        let mut key = String::new();
        key.push_str("key_");
        if value == 0 {
            key.push('0');
        } else {
            let mut temp = value;
            let mut digits = [0u8; 10];
            let mut len = 0;
            while temp > 0 {
                digits[len] = (temp % 10) as u8 + b'0';
                temp /= 10;
                len += 1;
            }
            for i in (0..len).rev() {
                key.push(digits[i] as char);
            }
        }
        assert_eq!(m.get(&key), Some(&value));
    }

    println!("test_hashmap() OK!");
}
