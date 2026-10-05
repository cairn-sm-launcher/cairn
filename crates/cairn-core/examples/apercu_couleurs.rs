//! Un apercu de ce que fait l'ecretage, pour juger sur pieces.
fn main() {
    use cairn_core::lisibilite::*;
    let fond = FOND_REFERENCE;
    println!("code      origine   contraste   corrige   contraste");
    for (code, c) in [
        ("$000", [0x00, 0x00, 0x00]),
        ("$111", [0x11, 0x11, 0x11]),
        ("$333", [0x33, 0x33, 0x33]),
        ("$005", [0x00, 0x00, 0x55]),
        ("$00f", [0x00, 0x00, 0xFF]),
        ("$300", [0x33, 0x00, 0x00]),
        ("$f00", [0xFF, 0x00, 0x00]),
        ("$080", [0x00, 0x88, 0x00]),
        ("$0f0", [0x00, 0xFF, 0x00]),
        ("$606", [0x66, 0x00, 0x66]),
        ("$fff", [0xFF, 0xFF, 0xFF]),
        ("$fc5", [0xFF, 0xCC, 0x55]),
    ] {
        let corrige = lisible(c);
        println!(
            "{code}      {}   {:>6.2}      {}   {:>6.2}{}",
            en_hexadecimal(c),
            contraste(c, fond),
            en_hexadecimal(corrige),
            contraste(corrige, fond),
            if corrige == c { "   inchange" } else { "" }
        );
    }
}
