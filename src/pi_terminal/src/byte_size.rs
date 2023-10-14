// https://github.com/hyunsik/bytesize

use bytesize::ByteSize;

pub fn mk_lib_common_bytesize(number_for_format: u64) -> Result<String, std::io::Error> {
    let result = ByteSize(number_for_format).to_string();
    Ok(result)
}
