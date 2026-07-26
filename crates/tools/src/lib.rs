pub mod json_formatter;
pub mod json_validator;
pub mod base64_codec;
pub mod timestamp;
pub mod uuid_gen;
pub mod url_codec;
pub mod text_stats;
pub mod string_case;
pub mod hash_tool;

/// 注册所有内置工具到 registry
pub fn register_all(registry: &mut trove_core::ToolRegistry) {
    registry
        .register(json_formatter::JsonFormatter)
        .register(json_validator::JsonValidator)
        .register(base64_codec::Base64Encode)
        .register(base64_codec::Base64Decode)
        .register(timestamp::TimestampToDate)
        .register(timestamp::DateToTimestamp)
        .register(uuid_gen::UuidGen)
        .register(url_codec::UrlEncode)
        .register(url_codec::UrlDecode)
        .register(text_stats::TextStats)
        .register(string_case::StringCase)
        .register(hash_tool::HashTool);
}
