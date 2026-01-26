use std::mem;

const DEFAULT_PATH: &str = "main.ml";
const DEFAULT_EMPTY_CHARS: &[char; 0] = &[];
const ESCAPES: [(char, char); 14] = [
    ('0', '\x00'), ('a', '\x07'),
    ('b', '\x08'), ('f', '\x0C'),
    ('n', '\n'),   ('r', '\r'),
    ('s', ' '),    ('t', '\t'),
    ('v', '\x0B'), ('\\', '\\'),   
    ('\"', '\"'),  ('{', '{'),
    ('\'', '\''),  ('}', '}'),
];

pub struct Group {
    pub body: String,
    pub pairs: Option<Pairs>
}

/// Chứa thông tin về hàng trong file.
/// 
/// Lines:
///     - `vector_char`: Tất cả ký tự của hàng đó trong file ở UTF-8.
///     - `length`: Độ dài của `vector_char`.
///     - `path`: Đường dẫn tới file, dùng trong debug.
///     - `line`: Nội dung của hàng ở dạng chuỗi.
///     - `rows`: Hàng hiện tại trong file, tính từ 0.
///     - `cols`: Vị trí ký tự hiện tại trong file, tính từ 0.
pub struct Lines {
    pub vector_char: Vec<char>,
    pub length: usize,
    pub path: String,
    pub line: String,
    pub rows: usize,
    pub cols: usize,
}

/// StringUTF8:
///     - `string_utf8`: Kết quả chuỗi UTF-8.
///     - `string_length`: Độ dài sau khi decode.
///     - `original_length`: Độ dài góc cuat toàn bộ chuỗi.
pub struct StringUTF8 {
    pub string_utf8: String,
    pub string_length: usize,
    pub original_length: usize,
}

/// ParseError:
///     - `status_code`: Mã lỗi duy nhất.
///     - `message`: Thông báo lỗi.
pub struct ParseError {
    pub status_code: usize,
    pub message: String,
}

#[derive(Clone, Copy)]
pub enum Pairs {
    Parentheses,
    Brackets,
    Braces,
    Angles
}

/// Dùng đề điều khiển vòng lặp gián tiếp. Dùng match để bắt các trường hợp
/// 
/// LoopControl:
///     - `Nothing`: Không làm gì cả, `LoopControl::Nothing => {}`
///     - `Break`: Dừng vòng lặp, `LoopControl::Break => break`
///     - `Continue`: Bỏ qua một bước, `LoopControl::Continue => continue`
///     - `Return`: Yêu cầu trả về giá trị, `LoopControl::Return => return something`
pub enum LoopControl {
    Nothing,
    Break,
    Continue,
    Return
}

impl Group {
    /// Hàm sẽ parse group đầu tiên mà nó thấy theo chỉ định của `pairs`.
    /// Có xử lý group lồng nhau, nhưng không đệ quy, chỉ lấy `body`.
    fn parse(lines: &mut Lines, pairs: Pairs) -> Result<Self, ParseError> {
        // Lấy dấu open và close tương ứng theo `pairs`
        let (open, close) = match pairs {
            Pairs::Angles => ('<', '>'),
            Pairs::Braces => ('{', '}'),
            Pairs::Brackets => ('[', ']'),
            Pairs::Parentheses => ('(', ')'),
        };

        // Group chứa `body` và loại dấu `pairs`
        let mut group = Self {
            body: String::new(),
            pairs: Some(pairs)
        };

        // Dùng để đếm độ lồng của group để xử lý
        let mut group_count: usize = 0;
        while lines.cols < lines.length {
            let current_char = lines.vector_char[lines.cols];

            // Xử lý trường hợp đặc biệt
            // Vì có thể có dấu `pairs` trong chuỗi gây sai lệch kết quả
            if current_char == '\"' && group_count != 0 {
                let sub_line = lines.slice_lines(lines.cols, None);         // Cắt từ ký tự `"` đến phần còn lại
                let mut sub_lines = Lines::new(None, &sub_line);                      // Tạo lines mới để có thể parse string
                let result = StringUTF8::parse(&mut sub_lines, false)?;
                let string = format!("\"{}\"", result.string_utf8);             // Dữ liệu có bao gồm dấu nháy
                group.body.push_str(&string);
                lines.cols += result.original_length + 2;       // Padding hai dấu nháy kép
                continue;
            }

            if current_char == open {
                // Kiểm tra đây là lần gặp đầu tiên
                // Vì chỉ lấy phần `body` không báo gồm phần bao bọc theo dấu `pairs`
                if group_count == 0 {
                    group_count += 1;
                    lines.cols += 1;
                    continue;
                }
                // Tăng giá trị khi lòng sâu
                group_count += 1;
            }

            if current_char == close {
                // Nếu gặp dấu `pairs` đóng khi chưa có dấu mở thì lỗi
                if group_count == 0 {
                    return Err(ParseError {
                        status_code: 5,
                        message: format!("Syntax Error: Invalid '{}' symbol!", current_char)
                    });
                }

                // Trừ rồi kiểm tra có hết `body` chưa, nếu có thì dừng
                group_count -= 1;
                if group_count == 0 {
                    lines.cols -= 1;    // Không bao gồm dấu `pairs` đóng vì chỉ lấy `body`
                    break;
                }
            }

            // Bỏ qua các ký tự thừa
            if group_count == 0 {
                lines.cols += 1;
                continue;
            }

            // Thêm ký tự của `body`
            group.body.push(current_char);
            lines.cols += 1;
        }

        Ok(group)
    }
    /// Hàm xử lý nhiều nhóm group cùng lúc. Trả về `Result<Vec<Self>, ParseError>`.
    /// Các nhóm Group có `Some(pairs)` là nhóm nằm trong cặp dấu `Pairs` tương ứng.
    /// Còn lại các nhóm có pairs là None thì là phần không nằm trong cặp dấu `Pairs` tương ứng.
    pub fn parse_groups(lines: &mut Lines, pairs: Pairs) -> Result<Vec<Self>, ParseError> {
        // Match dấu pairs tương ứng
        let open = match pairs {
            Pairs::Angles => '<',
            Pairs::Braces => '{',
            Pairs::Brackets => '[',
            Pairs::Parentheses => '('
        };
        let mut out_pairs = String::new();      // Biến tạm chứa nhóm không nằm trong cặp dấu `pairs`
        let mut vector_group: Vec<Self> = Vec::new();   // Biến chứa tất cả các nhóm

        while lines.cols < lines.length {
            let current_char = lines.vector_char[lines.cols];

            // Xử lý khi gặp dấu `pairs` tương ứng
            if current_char == open {
                let group = Self::parse(lines, pairs)?; // Ném lỗi nếu parse thất bại

                // Thêm phần bên ngoài `pairs` trước
                if !out_pairs.is_empty() {
                    let not_group = Self {
                        body: mem::take(&mut out_pairs), // Di chuyển và dọn dữ liệu cũ
                        pairs: None
                    };
                    vector_group.push(not_group);
                }
                vector_group.push(group);
                lines.cols += 2;    // Padding hai dấu `()`
                continue;
            }

            // Thêm ký tự bên ngoài `pairs`
            out_pairs.push(current_char);
            lines.cols += 1;
        }

        // Thêm phần còn lại nếu có
        if !out_pairs.is_empty() {
            vector_group.push(Self {
                body: out_pairs,
                pairs: None
            });
        }

        Ok(vector_group)
    }
}

impl Lines {
    /// Khởi tạo một Lines và đặt default.
    /// Có thể thêm path cho tiện debug.
    /// Đảm bảo để lines.cols = 0 nếu muốn đọc toàn bộ chuỗi.
    pub fn new(path: Option<&str>, line: &str) -> Self {
        let vector_char: Vec<char> = line.chars().collect();
        let length = vector_char.len();
        Self {
            vector_char: vector_char,
            length: length,
            line: line.to_string(),
            path: path.unwrap_or(DEFAULT_PATH).to_string(),
            rows: 0,
            cols: 0
        }
    }
    /// Khởi tạo một Lines đầy đủ hơn với rows và cols xác định.
    /// Path bắt buộc để rõ ràng.
    pub fn from(path: &str, line: &str, rows: usize, cols: usize) -> Self {
        let vector_char: Vec<char> = line.chars().collect();
        let length = vector_char.len();
        Self {
            vector_char,
            length: length,
            path: path.to_string(),
            line: line.to_string(),
            rows: rows,
            cols: cols
        }
    }
    /// Hàm cắt &Lines từ vị trí start đến length, cắt nhiều nhất.
    /// Luôn trả về `String`, trả về `""` (empty) nếu start hoặc length quá lớn.
    fn slice_lines(&self, start: usize, length: Option<usize>) -> String {
        let length = if let None = length { self.length } else { length.unwrap() };
        let endstr = (start + length).min(self.length);
        self.vector_char.get(start..endstr)
            .unwrap_or(DEFAULT_EMPTY_CHARS)
            .iter().collect()
    }
}

impl ParseError {
    /// Hàm này sẽ không bao giờ trả về giá trị.
    /// Chỉ nên dùng để debug và thông báo lỗi.
    pub fn panic(self, lines: &Lines) -> ! {
        let region = "^".repeat(lines.length);
        let error_message = format!("{}", self.message);

        eprintln!("[Error {:04}] {}", self.status_code, lines.line);
        eprintln!("              {}", region);
        eprintln!("File error at {}:{}:{}", lines.path, lines.rows + 1, lines.cols + 1);

        panic!("{error_message}");
    }
}

impl StringUTF8 {
    /// Hàm cắt Vec<char> từ vị trí start đến length, cắt nhiều nhất.
    /// Luôn trả về `&[char]`, trả về `[]` nếu start hoặc length quá lớn.
    fn slice_vector_char(vector_char: &Vec<char>, start: usize, length: Option<usize>) -> &[char] {
        let veclen = vector_char.len();
        let length = if let None = length { veclen } else { length.unwrap() };
        let endstr = (start + length).min(veclen);
        vector_char.get(start..endstr)
              .unwrap_or(DEFAULT_EMPTY_CHARS)
    }
    /// Chọn nhiều ký tự thuộc hexadecimal nhất có thể.
    /// Trả về [] nếu ngay ký tự đầu tiên không phai hexadecimal.
    fn match_char_hexadecimal_vaild(hexadecimal: &[char]) -> &[char] {
        let mut start = 0;
        for (index, char) in hexadecimal.iter().enumerate() {
            let item = &(*char as u32);
            if !(0x30..=0x39).contains(item)
            && !(0x41..=0x5A).contains(item)
            && !(0x61..=0x7A).contains(item) {
                break;
            }
            start = index + 1;
        }
        &hexadecimal[0..start]          // Extract only hexadecimal characters
    }
    /// Chuyển chuỗi hexadecimal thành unicode.
    /// None nếu code point nằm trong [0xD800; 0xDFFF] hoặc (0x10FFFF; ...).
    fn hexadecimal_to_unicode(source: &str) -> Option<char> {
        let item = u32::from_str_radix(source, 16).unwrap();
        char::from_u32(item)
    }
    /// Kiểm tra tính hợp lệ của chuỗi.
    ///     - Chuyển đổi trạng thái trong và ngoài chuỗi.
    ///     - Báo hiệu khi nào dừng.
    ///     - Bỏ qua các ký tự tự ở ngoài chuỗi.
    ///     - Không cho phép chuỗi cắt dòng.
    ///     - Cho phép chạy tiếp nếu không có gì.
    fn audit_string(current_char: char, is_in_string: &mut bool, cols: &mut usize) -> LoopControl {
        if current_char == '\"' {
            *is_in_string = !*is_in_string;

            // Trường hợp gặp ký tự '\"' một lần nữa thì dừng
            if !*is_in_string {

                // Bỏ qua dấu nháy cuối cùng vì chỉ lấy body
                *cols -= 1;
                return LoopControl::Break
            }

            // Bỏ qua dấu nháy kép đầu
            *cols += 1;
            return LoopControl::Continue
        }

        // Nếu đang nằm ngoài chuỗi thì bỏ qua
        if !*is_in_string {
            *cols += 1;
            return LoopControl::Continue
        }

        // Không cho phép viết chuỗi nhiều dòng
        if *is_in_string && current_char == '\n' {
            return LoopControl::Return
        }

        // Cho phép luồng bên dưới chạy tiếp
        LoopControl::Nothing
    }

    pub fn parse(lines: &mut Lines, is_raw_string: bool) -> Result<Self, ParseError> {
        let mut strutf8 = Self {
            string_utf8: String::new(),
            string_length: 0,
            original_length: 0,
        };

        // `is_raw_string`: Chỉnh định giải, không ảnh hưởng quá nhiều.
        // `is_in_string`: Để nhận biết đang trong chuỗi hay không.

        let mut is_in_string = false;
        while lines.cols < lines.length {
            let current_char = lines.vector_char[lines.cols];

            // Điều khiển luồng
            match StringUTF8::audit_string(current_char, &mut is_in_string, &mut lines.cols) {
                LoopControl::Nothing => {},
                LoopControl::Break => break,
                LoopControl::Continue => continue,
                LoopControl::Return => {
                    return Err(ParseError {
                        status_code: 0,
                        message: "Syntax Error: Invalid newline character in the string!".to_string()
                    });
                }
            };

            if current_char == '\\' && lines.cols + 1 < lines.length && !is_raw_string {
                let next_char = lines.vector_char[lines.cols + 1];
                let option_escape = ESCAPES.iter().position(|x| x.0 == next_char);

                // Gặp escape đơn hợp lệ, escape ký tự control, escape ký tự thường
                if let Some(index_escape) = option_escape {
                    let escape = ESCAPES[index_escape].1;
                    strutf8.string_utf8.push(escape);
                    strutf8.original_length += 2;
                    strutf8.string_length += 1;
                    lines.cols += 2;
                    continue;
                }

                // Xử lý unicode
                if next_char == 'u' {
                    let hexadecimal = Self::slice_vector_char(&lines.vector_char, lines.cols + 2, Some(8));
                    let source: String = Self::match_char_hexadecimal_vaild(hexadecimal).iter().collect();
                    if source.is_empty() {
                        return Err(ParseError {
                            status_code: 1,
                            message: "Escape Error: Invalid \\u escape sequence!".to_string()
                        });
                    }

                    // Nếu nằm trong khoảng unicode hợp lệ
                    if let Some(ch) = Self::hexadecimal_to_unicode(&source) {
                        let len_esc = source.len() + 2;
                        strutf8.string_utf8.push(ch);
                        strutf8.original_length += len_esc;
                        strutf8.string_length += 1;
                        lines.cols += len_esc;
                        continue;
                    }

                    return Err(ParseError {
                        status_code: 2,
                        message: "Unicode Error: Unicode is within an invalid range!".to_string()
                    });
                }

                // Xử lý hexadecimal
                if next_char == 'x' {
                    let hexadecimal = Self::slice_vector_char(&lines.vector_char, lines.cols + 2, Some(2));
                    let source: String = Self::match_char_hexadecimal_vaild(hexadecimal).iter().collect();

                    // Chỉ cho \xXX nếu không phải thì là lỗi
                    if source.len() != 2 {
                        return Err(ParseError {
                            status_code: 3,
                            message: "Byte Error: Invalid \\x escape: expected 2 hex digits!".to_string()
                        });
                    }

                    // unwrap do chỉ ascii và luôn hợp lệ
                    let ch = Self::hexadecimal_to_unicode(&source).unwrap();
                    strutf8.string_utf8.push(ch);
                    strutf8.original_length += 4;
                    strutf8.string_length += 1;
                    lines.cols += 4;
                    continue;
                }

                // Có dấu escape nhưng ký tự escape không hợp lệ
                println!("Warning Error: Escape invaild!");
                println!("File warning at {}:{}:{}", lines.path, lines.rows + 1, lines.cols + 1);
            }

            // Các ký tự còn lại thêm bình thường
            strutf8.string_utf8.push(current_char);
            strutf8.original_length += 1;
            strutf8.string_length += 1;
            lines.cols += 1;
        }

        // Trường hợp không thấy dấu nháy kép đóng.
        if is_in_string {
            return Err(ParseError {
                status_code: 4,
                message: "Syntax Error: Missing quotation marks at the end!".to_string()
            });
        }

        // Khi tất cả hợp lệ
        Ok(strutf8)
    }
}