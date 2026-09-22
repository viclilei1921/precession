use rand::Rng;

/// RFC 4122 UUID v4，给业务表主键用。
pub fn new_uuid_v4() -> String {
  let mut bytes = [0u8; 16];
  rand::rng().fill_bytes(&mut bytes);

  // 设置 UUID v4 的版本号和变体号
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  let hex = hex::encode(bytes);
  format!(
    "{}-{}-{}-{}-{}",
    &hex[0..8],
    &hex[8..12],
    &hex[12..16],
    &hex[16..20],
    &hex[20..32]
  )
}
