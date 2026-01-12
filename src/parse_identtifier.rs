//! # Docstring for Identifier
//! 
//! # Identifier: "^\[a-zA-Z\]\[a-zA-Z0-9_ '\]*\[a-zA-Z0-9_\]$"
//!     Lưu ký dấu ' ' và '\'' sẽ được đơn giản đi

const _KEYWORDS: [&str; 16] = ["and", "break", "continue", "define", "else", "if", "import", "in", "is", "let", "not", "or", "return", "None", "True", "False"];
const _DIGITALS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_'];

/// Hàm kiểm tra ký tự unicode hợp lệ.
/// Các khoảng không hợp lệ bao gồm \[0xD800; 0xDFFF\] và \[0x110000; ...\]
/// # Examples
///     assert_eq!(is_unicode_valid(&0x41), true);
///     assert_eq!(is_unicode_valid(&0xFFFF), true);
///     assert_eq!(is_unicode_valid(&0xD800), false);
///     assert_eq!(is_unicode_valid(&0x110000), false);
fn is_unicode_valid(code: &u32) -> bool {
    let item: &u32 = code;

    return !(0xD800..=0xDFFF).contains(item)
    || *item <= 0x10FFFF;
}

/// Hàm kiểm tra ký tự identifier có hợp lệ không với mẫu [a-zA-Z0-9_ '\]
/// # Examples
///     assert_eq!(is_identifier(&('A' as u32)), true);
///     assert_eq!(is_identifier(&('z' as u32)), true);
///     assert_eq!(is_identifier(&('0' as u32)), true);
///     assert_eq!(is_identifier(&(' ' as u32)), true);
fn is_identifier(code: &u32) -> bool {
    let item: &u32 = code;

    return (0x30..=0x39).contains(item)
        || (0x41..=0x5A).contains(item)
        || (0x41..=0x7A).contains(item)
        || [0x20, 0x27, 0x5F].contains(item);
}

/// Từ trong tên biến và tên biến không được trùng với từ khóa và không đặt bắt đầu bằng sô hay gạch chân.
/// Lý do Option<T> vì identitify không hợp lệ ví dụ bắt đầu bằng ^\[0-9_\].
/// 
/// Hàm xử lý identifier, chấp chận dấu space và dấu nháy.
/// Hàm trả về một tuple lần lượt là tên biến, ký tự không phải identifier
/// hoặc từ khóa, độ dài gốc.
pub fn parse(line: &str) -> Option<(String, String, usize)> {
    let mut name: String = String::new();
    let mut word: String = String::new();
    let mut keyw: String = String::new();
    let mut chrn: String = String::new();
    let mut hssp: bool = false;
    let mut hsap: bool = false;
    let mut length: usize = 0;
    
    for (index, char) in line.chars().enumerate() {
        let code: &u32 = &(char as u32);

        // Bỏ tất cả ký tư identifier không hợp lệ
        if !is_unicode_valid(code) {
            chrn = char.to_string();
            length = index;
            break;
        }

        if !is_identifier(code) {
            chrn = char.to_string();
            length = index;
            break;
        }

        // Bỏ qua các ký tự thừa
        if [0x20, 0x27].contains(code) && name.is_empty() && word.is_empty() {
            continue;
        }

        // Xử lý dấu nháy
        if *code == 0x27 {
            hsap = true;

            if word.is_empty() {
                continue;
            }

            // Nếu trùng từ khóa
            if _KEYWORDS.contains(&&*word) {
                // Bị chuyển quyền sỡ hũu
                keyw = word.clone();
                length = index;
                break;
            }

            // Thêm vào tên biến
            let string: &String = &word;
            name.push_str(string);
            word.clear();
            continue;
        }

        // Logic tương tự
        if *code == 0x20 {
            hssp = true;

            if word.is_empty() {
                continue;
            }

            if _KEYWORDS.contains(&&*word) {
                keyw = word.clone();
                length = index;
                break;
            }

            let string: &String= &word;
            name.push_str(string);
            word.clear();
            continue;
        }

        // Thêm dấu nháy và thêm dấu space
        if hsap {
            let ch: char = '\x27';
            name.push(ch);
            hsap = false;
        } else
        if hssp {
            let ch: char = '\x20';
            name.push(ch);
            hssp = false;
        }

        let ch: char = char;
        word.push(ch);
        length = index;
    }

    // Nếu còn từ và không có keyword thì thêm vào tên biến
    if !word.is_empty() && keyw.is_empty() {
        let string: &String = &word;
        name.push_str(string);
    }

    // Kiểm tra tính hợp lệ
    if name.starts_with(&_DIGITALS) {
        return None;
    }

    // Thêm ký tự không phải identifier bao gồm cả từ khóa
    let sepr: String = if !chrn.is_empty() { chrn } else { keyw };
    let result: (String, String, usize) = (name.trim_matches(&['\x20', '\x27']).to_string(), sepr, length);

    return Some(result);
}