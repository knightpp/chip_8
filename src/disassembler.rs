use crate::word_to_nibbles;
use crate::MergeNibbles;

pub fn disassemble_file(mut file: impl std::io::Read) -> String {
    let mut instruction = [0_u8; 2];
    let mut buf = String::new();
    let mut total_read = 0;
    let mut i = 1_usize;
    loop {
        let read = file.read(&mut instruction).unwrap();
        if read == 0 {
            break;
        } else if read == 1 {
            panic!();
        }
        buf.push_str(&format!("{:02}) {:04X}\t", i, total_read));
        match word_to_nibbles(&instruction) {
            // FX65
            [0xF, x, 0x6, 0x5] => {
                buf.push_str(&format!("reg_load(V{:1X}, &I)", x));
            }
            // FX55
            [0xF, x, 0x5, 0x5] => {
                buf.push_str(&format!("reg_dump(V{:1X}, &I)", x));
            }
            // FX33
            [0xF, x, 0x3, 0x3] => {
                buf.push_str(&format!("set_BCD(V{:1X})", x));
            }
            // FX29
            [0xF, x, 0x2, 0x9] => {
                buf.push_str(&format!("I = sprite_addr(V{:1X})", x));
            }
            // FX1E
            [0xF, x, 1, 0xE] => {
                buf.push_str(&format!("I += V{:1X}", x));
            }
            // FX18
            [0xF, x, 1, 0x8] => {
                buf.push_str(&format!("sound_timer(V{:1X})", x));
            }
            // FX15
            [0xF, x, 1, 0x5] => {
                buf.push_str(&format!("delay_timer(V{:1X})", x));
            }
            // FX0A
            [0xF, x, 0, 0xA] => {
                buf.push_str(&format!("V{:1X}=get_key()", x));
            }
            // FX07
            [0xF, x, 0, 0x7] => {
                buf.push_str(&format!("V{:1X}=get_delay()", x));
            }
            // EXA1
            [0xE, x, 0xA, 0x1] => {
                buf.push_str(&format!("if(key()!=V{:1X})", x));
            }
            // EX9E
            [0xE, x, 0x9, 0xE] => {
                buf.push_str(&format!("if(key()==V{:1X})", x));
            }
            // DXYN
            [0xD, x, y, n] => {
                buf.push_str(&format!("draw(V{:1X}, V{:1X}, 0x{:1X})", x, y, n));
            }
            // CXNN
            [0xC, x, nn @ ..] => {
                buf.push_str(&format!("V{:1X}=rand() & 0x{:02X}", x, nn.merge_nibbles()));
            }
            // BNNN
            [0xB, nnn @ ..] => {
                buf.push_str(&format!("PC=V0+0x{:03X}", nnn.merge_nibbles()));
            }
            // ANNN
            [0xA, nnn @ ..] => {
                buf.push_str(&format!("I=0x{:03X}", nnn.merge_nibbles()));
            }
            // 9XY0
            [0x9, x, y, 0] => {
                buf.push_str(&format!("if(V{:1X}!=V{:1X}) skip_next;", x, y));
            }
            // 8XY7
            [0x8, x, y, 0x7] => {
                buf.push_str(&format!("V{:1X}=V{:1X}-V{:1X}", x, y, x));
            }
            // 8XY6
            [0x8, x, _y, 0x6] => {
                buf.push_str(&format!("V{:1X}>>=1", x));
            }
            // 8XY5
            [0x8, x, y, 0x5] => {
                buf.push_str(&format!("V{:1X}-=V{:1X}", x, y));
            }
            // 8XY4
            [0x8, x, y, 0x4] => {
                buf.push_str(&format!("V{:1X}+=V{:1X}", x, y));
            }
            // 8XY3
            [0x8, x, y, 0x3] => {
                buf.push_str(&format!("V{:1X}=V{:1X}^V{:1X}", x, x, y));
            }
            // 8XY2
            [0x8, x, y, 0x2] => {
                buf.push_str(&format!("V{:1X}=V{:1X}&V{:1X}", x, x, y));
            }
            // 8XY1
            [0x8, x, y, 1] => {
                buf.push_str(&format!("V{:1X}=V{:1X}|V{:1X}", x, x, y));
            }
            // 8XY0
            [0x8, x, y, 0] => {
                buf.push_str(&format!("V{:1X}=V{:1X}", x, y));
            }
            // 8XYE
            [0x8, x, _y, _e] => {
                buf.push_str(&format!("V{:1X}<<=1", x));
            }
            // 7XNN
            [0x7, x, nn @ ..] => {
                buf.push_str(&format!("V{:1X}+={:02X}", x, nn.merge_nibbles()));
            }
            // 6XNN
            [0x6, x, nn @ ..] => {
                buf.push_str(&format!("V{:1X}={:02X}", x, nn.merge_nibbles()));
            }
            // 5XY0
            [0x5, x, y, 0] => {
                buf.push_str(&format!("if(V{:1X}==V{:1X}) skip_next;", x, y));
            }
            // 4XNN
            [0x4, x, nn @ ..] => {
                buf.push_str(&format!(
                    "if(V{:1X}!={:02X}) skip_next;",
                    x,
                    nn.merge_nibbles()
                ));
            }
            // 3XNN
            [0x3, x, nn @ ..] => {
                buf.push_str(&format!(
                    "if(V{:1X}=={:02X}) skip_next;",
                    x,
                    nn.merge_nibbles()
                ));
            }
            // 2NNN
            [0x2, nnn @ ..] => {
                buf.push_str(&format!("{} {:03X};", "call", nnn.merge_nibbles()));
            }
            // 1NNN
            [1, nnn @ ..] => {
                buf.push_str(&format!("{} {:03X};", "goto", nnn.merge_nibbles()));
            }
            // 00EE
            [0, 0, 0xE, 0xE] => {
                buf.push_str("return;");
            }
            // 00E0
            [0, 0, 0xE, 0] => {
                buf.push_str("disp_clear");
            }
            // 0NNN Call
            [0, nnn @ ..] => {
                buf.push_str(&format!("{} {:03X}", "call", nnn.merge_nibbles()));
            }
            left => {
                //panic!("unknown instruction");
                buf.push_str(&format!("UNKNOWN INSTRUCTION {:1X?}", left,));
            }
        }
        buf.push('\n');

        total_read += read;
        i += 1;
    }

    buf
}
