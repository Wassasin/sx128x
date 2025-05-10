use crate::hl::Frequency;

const fn freq_reference(freq: u32) -> [u8; 3] {
    let val = freq as f64 / 198.3642578125;
    let val = val as u32;
    [(val >> 16) as u8, (val >> 8) as u8, val as u8]
}

#[test]
fn frequency() {
    assert_eq!(
        Frequency::new(2_400_000_000).as_bytes(),
        freq_reference(2_400_000_000)
    );
    assert_eq!(
        Frequency::new(2_440_000_000).as_bytes(),
        freq_reference(2_440_000_000)
    );
    assert_eq!(
        Frequency::new(2_485_000_000).as_bytes(),
        freq_reference(2_485_000_000)
    );
}
