const _HEXADECIMAL: u32 = 16;
const _MAP_ESCAPES: [u32; 16] = [0x30, 0x00, 0x61, 0x07, 0x62, 0x08, 0x66, 0x0C, 0x6E, 0x0A, 0x72, 0x0D, 0x74, 0x09, 0x76, 0x0B];
const _ESCAPE_CHARS: [u32; 5] = [0x5C, 0x22, 0x27, 0x7B, 0x7D];

/// Cắt chuỗi từ ký tự tại vị trí chỉ định với kích thước length.
/// Hàm sẽ cắt nhiều nhất có thể.
/// # Examples
///     let chars: &Vec<char> = &['H', 'e', 'l', 'l', 'o', ',', ' ', 'w', 'o', 'r', 'l', 'd', '!', '\n'].to_vec();
///     let lenvc = chars.len();
/// 
///     let result = _slice_vector_chars(chars, lenvc, 0, 14);
///     assert_eq!(result, "Hello, world\n".to_string())
/// 
///     let result = _slice_vector_chars(chars, lenvc, 7, 12);
///     assert_eq!(result, "world".to_string())
/// 
///     let result = _slice_vector_chars(chars, lenvc, 0, 100);
///     assert_eq!(result, "Hello, world\n".to_string())
/// 
///     let result = _slice_vector_chars(chars, lenvc, 100, 150);
///     assert_eq!(result, "")
/// 
///     let result = _slice_vector_chars(chars, lenvc, 100, 50);
///     assert_eq!(result, "")
pub fn slice_vector_chars(vector_char: &Vec<char>, vector_length: usize, start: usize, length: usize) -> String {
    let mut len: usize = length;
    if len == 0 {
        len = vector_length;
    }
    let end: usize = (start + len).min(vector_length);
    let index: std::ops::Range<usize> = start..end;
    let default: &[char; 0] = &[];
    let sub: String = vector_char.get(index).unwrap_or(default).iter().collect();
    return sub;
}

/// Hàm sẽ lấy nhiều ký tự nhất trong hệ hexadecimal, không phân biệt chữ hoa thường và chỉ ascii.
/// # Examples
///     let hexadecimal: &str = "0123456789abcdefABCDEF";
///     let result: String = _get_chars_hexadecimal_vaild(hexadecimal);
///     assert_eq!(result, "0123456789abcdefABCDEF".to_string())
/// 
///     let hexadecimal: &str = "2a3es303f";
///     let result: String = _get_chars_hexadecimal_vaild(hexadecimal);
///     assert_eq!(result, "2a3e".to_string())
/// 
///     let hexadecimal: &str = "kde3jd";
///     let result: String = _get_chars_hexadecimal_vaild(hexadecimal);
///     assert_eq!(result, "".to_string())
fn _get_chars_hexadecimal_vaild(hexadecimal: &str) -> String {
    let mut hex: String = String::new();
    for char in  hexadecimal.to_ascii_uppercase().chars() {
        let item: &u32 = &(char as u32);
        if (0x30..=0x39).contains(item)         // Chỉ nhận ký tự trong hệ hex
        || (0x41..=0x5A).contains(item) {
            let ch: char = char;
            hex.push(ch);
            continue;
        }
        break;      // Khi gặp ký tự không hợp lệ
    }
    return hex;
}

/// Chuyển chuỗi ký tự hệ hexadecimal thành một ký tự unicode, có thể xảy ra lỗi.
/// Nó sẽ là nội bộ vì hàm không an toàn.
/// 
/// Cần kiểm tra chỉ 8 ký tự hexadecimal (Có thể padding ký tự '0') và chỉ ký tự hexadecimal hợp lệ.
/// # Examples
///     let src: &str = "41";
///     let result: Option<char> = _hexadecimal_to_unicode(src); 
///     assert_eq!(Some(result), 'A');
/// 
///     let src: &str = "0041";
///     let result: Option<char> = _hexadecimal_to_unicode(src); 
///     assert_eq!(Some(result), 'A');
/// 
///     let src: &str = "000000000041";
///     let result: Option<char> = _hexadecimal_to_unicode(src); 
///     assert_eq!(Some(result), 'A');
fn _hexadecimal_to_unicode(src: &str) -> Option<char> {
    let radix: u32 = _HEXADECIMAL;
    let item: &u32 = &u32::from_str_radix(src, radix).unwrap();
    if !(0xD800..=0xDFFF).contains(item) && *item < 0x110000 {
        let char: char = char::from_u32(*item).unwrap();
        return Some(char);
    }
    return None;
}

/// Điều khiển luồng từ hàm bên ngoài.
/// Sẽ dùng match để bắt các trường hợp.
/// Dùng `CtrlLoop::Nothing => {}` để luồng phía dưới có thể tiếp tục chạy.
enum CtrlLoop {
    Continue,
    Break,
    Return,
    Nothing
}

pub struct NewString {
    pub new_string: String,
    pub string_length: usize,
    pub original_length: usize
}

/// Hàm điều khiển luồng vào lúc tiền xử lý chuỗi.
/// Khi còn gặp ký tự `"` cho trạng thái trong chuỗi, khi gặp lần hai được xem là kết thúc.
/// Khi còn ngoài chuỗi thì tiếp tục bỏ qua.
/// Không cho phép chuỗi xuông dòng sẽ trả về `CtrlLoop::Return`.
/// Nếu vượt qua tất cả thì cho luồng tiếp tục chạy phần code kết tiếp.
fn _audit_string(current_code: u32, is_in_string: &mut bool, index: &mut usize) -> CtrlLoop {
    if current_code == 0x22 {
        *is_in_string = !*is_in_string;

        // Trường hợp gặp ký tự '\"' một lần nữa thì dừng
        if !*is_in_string {
            // Bỏ qua dấu nháy cuối cùng
            *index -= 1;
            return CtrlLoop::Break;
        }

        // Bỏ qua dấu nháy kép đầu
        *index += 1;
        return CtrlLoop::Continue;
    }

    // Nếu đang nằm ngoài chuỗi thì bỏ qua
    if !*is_in_string {
        *index += 1;
        return CtrlLoop::Continue;
    }

    // Không cho phép viết chuỗi nhiều dòng
    if *is_in_string && current_code == 0x0A {
        return CtrlLoop::Return;
    }

    // Cho phép luồng bên dưới chạy tiếp
    return CtrlLoop::Nothing;
}

/// Hàm xử lý chuỗi bao gồm escape.
/// # Rules
/// Chỉ nhận chuỗi sử dụng dấu nháy kép `"` và các escape bắt đầu bắt ký tự '\\'.
/// Xử lý escape đơn chỉ nhận loại escape: \0, \a, \b, \f, \n, \r, \t, \v.
/// 
/// Escape hexacadecimal chỉ a-fA-F0-9, bắt buộc tối đa 2 ký tự.
/// Escape unicode chỉ nhận a-fA-F0-9, độ dài ít nhất 1 đến tối đa 8 ký tự.
/// Nếu escape không hợp lệ như \c thì hàm sẽ cho nó là \\\c và cảnh báo.
/// Không xử lý octal.
/// 
/// Chuỗi không được xuống dòng, hàm chỉ xử lý chuỗi đầu tiên.
pub fn parse(path: &str, line: &str, rows: &usize) -> Option<NewString> {
    let vector_char: Vec<char> = line.chars().collect();
    let vector_length: usize = vector_char.len();
    let mut is_in_string: bool = false;
    let mut new_string: String = String::new();
    let mut string_length: usize = 0;   // Độ dài chuỗi sau khi decode
    let mut original_length: usize = 0; // Độ dài góc chủa chuỗi
    let mut index: usize = 0;

    while index < vector_length {
        let current_char: char = vector_char[index];
        let current_code: u32 = current_char as u32;

        // Điều khiển luồng
        match _audit_string(current_code, &mut is_in_string, &mut index) {
            CtrlLoop::Continue => continue,
            CtrlLoop::Break => break,
            CtrlLoop::Return => {
                println!("Syntax Error: Invalid newline character in the string!");
                println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
                return None;
            }
            CtrlLoop::Nothing => {}
        };

        if current_code == 0x5C && index + 1 < vector_length {
            let next_char: char = vector_char[index + 1];
            let next_code: u32 = next_char as u32;
            let start: usize = index + 2;

            // Gặp escape đơn hợp lệ, escape ký tự control
            if _MAP_ESCAPES.contains(&next_code) {
                let index_escape: usize = _MAP_ESCAPES.iter().position(|v| v == &next_code).unwrap();
                let ch: char = _MAP_ESCAPES[index_escape + 1] as u8 as char;
                new_string.push(ch);
                original_length += 2;
                string_length += 1;
                index += 2;
                continue;
            }

            // Gặp loại escape đơn hợp lệ, escape ký tự thường
            if _ESCAPE_CHARS.contains(&next_code) {
                let ch: char = next_char;
                new_string.push(ch);
                original_length += 2;
                string_length += 1;
                index += 2;
                continue;
            }

            // Xử lý unicode
            if next_code == 0x75 {
                let length: usize = 8;
                let hexadecimal: String = slice_vector_chars(&vector_char, vector_length, start, length);
                let src: String = _get_chars_hexadecimal_vaild(&hexadecimal);
                if src.is_empty() {
                    println!("Escape Error: Invalid \\u escape sequence!");
                    println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
                    return None;
                }

                // Nếu nằm trong khoảng unicode hợp lệ
                if let Some(ch) = _hexadecimal_to_unicode(&src) {
                    new_string.push(ch);
                    let escape_length: usize = src.len() + 2;
                    original_length += escape_length;
                    index += escape_length;
                    string_length += 1;
                    continue;
                }

                println!("Unicode Error: Unicode is within an invalid range!");
                println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
                return None;
            }

            // Xử lý hexadecimal
            if next_code == 0x78 {
                let length: usize = 2;
                let hexadecimal: String = slice_vector_chars(&vector_char, vector_length, start, length);
                let src: String = _get_chars_hexadecimal_vaild(&hexadecimal);

                // Chỉ cho \xXX nếu không phải thì lỗi
                if src.len() != 2 {
                    println!("Byte Error: Invalid \\x escape: expected 2 hex digits!");
                    println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
                    return None;
                }

                // unwrap do chỉ ascii và luôn hợp lệ
                let ch: char = _hexadecimal_to_unicode(&src).unwrap();
                new_string.push(ch);
                original_length += 4;
                string_length += 1;
                index += 4;
                continue;
            }

            println!("Warning Error: Escape invaild!");
            println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
        }

        // Các ký tự còn lại thêm binhg thường
        let ch: char = current_char;
        new_string.push(ch);
        original_length += 1;
        string_length += 1;
        index += 1;
    }

    // Trường hợp không thấy dấu nháy kép đóng.
    if is_in_string {
        println!("Syntax Error: Missing quotation marks at the end!");
        println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
        return None;
    }

    // Khi tất cả hợp lệ
    let result: NewString = NewString {new_string, string_length, original_length};
    return Some(result);
}

/// Hàm xử lý chuỗi chỉ bao gồm escape `"`.
/// Các thành phần còn lại không được escape.
pub fn parse_raw(path: &str, line: &str, rows: &usize) -> Option<NewString> {
    let vector_chars: Vec<char> = line.chars().collect();
    let vector_length: usize = vector_chars.len();
    let mut is_in_string: bool = false;
    let mut new_string: String = String::new();
    let mut string_length: usize = 0;
    let mut original_length: usize = 0;
    let mut index: usize = 0;

    while index < vector_length {
        let current_char: char = vector_chars[index];
        let current_code: u32 = current_char as u32;

        match _audit_string(current_code, &mut is_in_string, &mut index) {
            CtrlLoop::Continue => continue,
            CtrlLoop::Break => break,
            CtrlLoop::Return => {
                println!("Syntax Error: Invalid newline character in the string!");
                println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
                return None;
            }
            CtrlLoop::Nothing => {}
        };

        if current_code == 0x5C && index + 1 < vector_length {
            let next_char: char = vector_chars[index + 1];
            let next_code: u32 = next_char as u32;
            // Chỉ xử lý escape `"`
            if next_code == 0x22 {
                let ch: char = next_char;
                new_string.push(ch);
                original_length += 2;
                string_length += 1;
                index += 2;
                continue;
            }
        }

        // Các ký tự còn lại sẽ thêm vào bình thường
        let ch: char = current_char;
        new_string.push(ch);
        original_length += 1;
        string_length += 1;
        index += 1;
    }

    if is_in_string {
        println!("Syntax Error: Missing quotation marks at the end!");
        println!("File error at {}:{}:{}.", path, rows + 1, index + 1);
        return None;
    }

    let result: NewString = NewString {new_string, string_length, original_length};
    return Some(result);
}