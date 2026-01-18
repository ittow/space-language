use crate::parse_string;
use crate::parse_string::NewString;

/// Hàm xử lý nhóm, hoặc các dấu ngoặc nói chung.
/// Chỉ bắt nhóm đầu tiên, phần xử lý ký tự phân cách không nằm trong hàm này.
/// Hàm sẽ trả về chuỗi của nhóm kèm độ dài.
pub fn parse(path: &str, line: &str, rows: &usize) -> Option<(String, usize)> {
    let vector_char: Vec<char> = line.chars().collect();
    let vector_length: usize = vector_char.len();
    let mut block: Vec<u32> = Vec::new();
    let mut start: Option<usize> = None;
    let mut end  : Option<usize> = None;
    let mut index: usize = 0;
    while index < vector_length {
        let current_char: char = vector_char[index];
        let current_code: u32 = current_char as u32;
        let length: usize = block.len();
        match current_code {
            0x28 | 0x5B | 0x7B => {
                block.push(current_code);
                if let None = start {
                    // Bỏ qua ký tự đầu tiên của nhóm
                    start = Some(index + 1);
                }
            }

            // Xử lý các dấu đóng xem chúng có nằm ở vị trí hợp lệ
            // Và xử lý trường hợp dấu đóng trong chuỗi
            0x29 => {
                if length > 0 && block[length - 1] == 0x28 {
                    block.pop();
                    if block.is_empty() {
                        end = Some(index);
                        break;
                    }
                } else {
                    println!("Syntax Error: The ')' symbol is in an invalid position!");
                    println!("File error at {}:{}:{}", path, rows + 1, index + 1);
                    return None;
                }
            }
            0x5D => {
                if length > 0 && block[length - 1] == 0x5B {
                    block.pop();
                    if block.is_empty() {
                        end = Some(index);
                        break;
                    }
                } else {
                    println!("Syntax Error: The ']' symbol is in an invalid position!");
                    println!("File error at {}:{}:{}", path, rows + 1, index + 1);
                    return None;
                }
            }
            0x7D => {
                if length > 0 && block[length - 1] == 0x7B {
                    block.pop();
                    if block.is_empty() {
                        end = Some(index);
                        break;
                    }
                } else {
                    println!("Syntax Error: The '}}' symbol is in an invalid position!");
                    println!("File error at {}:{}:{}", path, rows + 1, index + 1);
                    return None;
                }
            }
            0x22 => {
                // Xử lý chuỗi để bắt chính xác nhóm
                let sub_line: String = parse_string::slice_vector_chars(&vector_char, vector_length, index, 0);
                let result: Option<NewString> = parse_string::parse(path, &sub_line, rows);
                if let Some(value) = result {
                    let lenorg: usize = value.original_length;
                    index += lenorg + 2;    // Padding thêm hai dấu nháy kép "..."
                    continue;
                }
                return None;
            }

            // Các ký tự còn lại không được xử lý ở đây
            _ => {}
        };
        index += 1;
    }

    return match start {
        Some(s) => match end {
            Some(e) => {
                let start: usize = s;
                let length: usize = e - start;
                let result: String = parse_string::slice_vector_chars(&vector_char, vector_length, start, length);
                return Some((result, length));
            }
            None => None
        }
        None => None
    };
}