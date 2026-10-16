//! Thumb (16-bit) instruction encoding.

use super::{Context, condition, immediate, register, register_list, split_operands};

const ALU: &[&str] = &[
    "and", "eor", "lsl", "lsr", "asr", "adc", "sbc", "ror", "tst", "neg", "cmp", "cmn", "orr",
    "mul", "bic", "mvn",
];

pub fn is_conditional_branch(mnemonic: &str) -> bool {
    mnemonic.len() == 3
        && mnemonic.starts_with('b')
        && mnemonic != "bal"
        && condition(&mnemonic[1..]).is_some_and(|c| c != 14)
}

pub fn size(mnemonic: &str, _operands: &str, relaxed: bool) -> Result<u32, String> {
    if mnemonic == "bl" {
        return Ok(4);
    }
    if is_conditional_branch(mnemonic) && relaxed {
        return Ok(4);
    }
    Ok(2)
}

fn low(text: &str) -> Result<u32, String> {
    let r =
        register(text).ok_or_else(|| format!("expected a register, found `{}`", text.trim()))?;
    if r > 7 {
        return Err(format!(
            "`{}` must be a low register (r0-r7) in Thumb code",
            text.trim()
        ));
    }
    Ok(r)
}

fn any(text: &str) -> Result<u32, String> {
    register(text).ok_or_else(|| format!("expected a register, found `{}`", text.trim()))
}

fn imm(ctx: &Context, text: &str, max: i64, scale: i64) -> Result<u32, String> {
    let t =
        immediate(text).ok_or_else(|| format!("expected an immediate, found `{}`", text.trim()))?;
    let v = ctx.eval(t)?;
    if v % scale != 0 {
        return Err(format!("immediate {v} must be a multiple of {scale}"));
    }
    if v < 0 || v > max {
        return Err(format!("immediate {v} is out of range (0..={max})"));
    }
    Ok((v / scale) as u32)
}

fn half(word: u32) -> Vec<u8> {
    (word as u16).to_le_bytes().to_vec()
}

/// Strips a UAL flag-setting `s` suffix from known mnemonics.
fn base(mnemonic: &str) -> &str {
    const FLAGGED: &[&str] = &[
        "movs", "adds", "subs", "ands", "eors", "lsls", "lsrs", "asrs", "adcs", "sbcs", "rors",
        "negs", "orrs", "muls", "bics", "mvns", "rsbs",
    ];
    if FLAGGED.contains(&mnemonic) {
        &mnemonic[..mnemonic.len() - 1]
    } else {
        mnemonic
    }
}

pub fn encode(
    mnemonic: &str,
    operands: &str,
    ctx: &Context,
    relaxed: bool,
) -> Result<(Vec<u8>, Option<u32>), String> {
    let ops = split_operands(operands);
    let m = base(mnemonic);
    let op = |i: usize| -> Result<&str, String> {
        ops.get(i)
            .map(String::as_str)
            .ok_or_else(|| format!("`{mnemonic}` is missing an operand"))
    };
    let here = ctx.here() as i64;
    let code = match m {
        "nop" => half(0x46C0),
        "mov" => {
            let rd = any(op(0)?)?;
            if let Some(i) = immediate(op(1)?).filter(|_| register(op(1).unwrap()).is_none()) {
                if rd > 7 {
                    return Err("mov #imm needs a low destination register".into());
                }
                let v = ctx.eval(i)?;
                if !(0..=255).contains(&v) {
                    return Err(format!(
                        "mov immediate {v} is out of range (0..=255); use `ldr rX, ={v}`"
                    ));
                }
                half(0x2000 | (rd << 8) | v as u32)
            } else {
                let rs = any(op(1)?)?;
                if rd < 8 && rs < 8 {
                    half((rs << 3) | rd)
                } else {
                    half(0x4600 | ((rd & 8) << 4) | (rs << 3) | (rd & 7))
                }
            }
        }
        "cmp" => {
            let rd = any(op(0)?)?;
            if let Some(i) = immediate(op(1)?).filter(|_| register(op(1).unwrap()).is_none()) {
                if rd > 7 {
                    return Err("cmp #imm needs a low register".into());
                }
                let v = ctx.eval(i)?;
                if !(0..=255).contains(&v) {
                    return Err(format!("cmp immediate {v} is out of range (0..=255)"));
                }
                half(0x2800 | (rd << 8) | v as u32)
            } else {
                let rs = any(op(1)?)?;
                if rd < 8 && rs < 8 {
                    half(0x4280 | (rs << 3) | rd)
                } else {
                    half(0x4500 | ((rd & 8) << 4) | (rs << 3) | (rd & 7))
                }
            }
        }
        "add" | "sub" => {
            let sub = m == "sub";
            let rd = any(op(0)?)?;
            if ops.len() == 3 {
                let rs = any(op(1)?)?;
                if let Some(i) = immediate(op(2)?).filter(|_| register(op(2).unwrap()).is_none()) {
                    let v = ctx.eval(i)?;
                    if rs == 13 && rd == 13 {
                        let v = if sub { -v } else { v };
                        return Ok((sp_adjust(v)?, None));
                    }
                    if (rs == 13 || rs == 15) && !sub {
                        if rd > 7 {
                            return Err("add rd, sp/pc needs a low destination".into());
                        }
                        if v % 4 != 0 || !(0..=1020).contains(&v) {
                            return Err(format!("offset {v} must be a multiple of 4 in 0..=1020"));
                        }
                        let opcode = if rs == 13 { 0xA800 } else { 0xA000 };
                        return Ok((half(opcode | (rd << 8) | (v as u32 / 4)), None));
                    }
                    let (rd, rs) = (low(op(0)?)?, low(op(1)?)?);
                    if (0..=7).contains(&v) {
                        half(
                            (if sub { 0x1E00 } else { 0x1C00 })
                                | ((v as u32) << 6)
                                | (rs << 3)
                                | rd,
                        )
                    } else if rd == rs && (0..=255).contains(&v) {
                        half((if sub { 0x3800 } else { 0x3000 }) | (rd << 8) | v as u32)
                    } else {
                        return Err(format!(
                            "immediate {v} is out of range for a three-operand {m}"
                        ));
                    }
                } else {
                    let rn = any(op(2)?)?;
                    if rd < 8 && rs < 8 && rn < 8 {
                        half((if sub { 0x1A00 } else { 0x1800 }) | (rn << 6) | (rs << 3) | rd)
                    } else if !sub && rd == rs {
                        half(0x4400 | ((rd & 8) << 4) | (rn << 3) | (rd & 7))
                    } else {
                        return Err(format!("unsupported register combination for {m}"));
                    }
                }
            } else {
                let second = op(1)?;
                if let Some(i) = immediate(second).filter(|_| register(second).is_none()) {
                    let v = ctx.eval(i)?;
                    if rd == 13 {
                        let v = if sub { -v } else { v };
                        return Ok((sp_adjust(v)?, None));
                    }
                    let rd = low(op(0)?)?;
                    let (sub, v) = if v < 0 { (!sub, -v) } else { (sub, v) };
                    if !(0..=255).contains(&v) {
                        return Err(format!("immediate {v} is out of range (0..=255)"));
                    }
                    half((if sub { 0x3800 } else { 0x3000 }) | (rd << 8) | v as u32)
                } else {
                    let rs = any(second)?;
                    if rd < 8 && rs < 8 {
                        half((if sub { 0x1A00 } else { 0x1800 }) | (rs << 6) | (rd << 3) | rd)
                    } else if !sub {
                        half(0x4400 | ((rd & 8) << 4) | (rs << 3) | (rd & 7))
                    } else {
                        return Err("sub with high registers is not available in Thumb".into());
                    }
                }
            }
        }
        "lsl" | "lsr" | "asr"
            if ops.len() == 3
                || ops
                    .get(1)
                    .is_some_and(|o| immediate(o).is_some() && register(o).is_none()) =>
        {
            let rd = low(op(0)?)?;
            let (rs, amount) = if ops.len() == 3 {
                (low(op(1)?)?, op(2)?)
            } else {
                (rd, op(1)?)
            };
            if let Some(i) = immediate(amount).filter(|_| register(amount).is_none()) {
                let v = ctx.eval(i)?;
                let kind = match m {
                    "lsl" => 0,
                    "lsr" => 1,
                    _ => 2,
                };
                let encoded = match (kind, v) {
                    (0, 0..=31) => v as u32,
                    (1 | 2, 1..=31) => v as u32,
                    (1 | 2, 32) => 0,
                    _ => return Err(format!("shift amount {v} is out of range")),
                };
                half((kind << 11) | (encoded << 6) | (rs << 3) | rd)
            } else {
                // lsl rd, rd, rs
                if rd != rs {
                    return Err("register shifts need the destination as the first source".into());
                }
                let r = low(amount)?;
                let index = ALU.iter().position(|a| *a == m).unwrap() as u32;
                half(0x4000 | (index << 6) | (r << 3) | rd)
            }
        }
        "rsb" => {
            // rsbs rd, rs, #0 == neg rd, rs
            let rd = low(op(0)?)?;
            let rs = low(op(1)?)?;
            half(0x4000 | (9 << 6) | (rs << 3) | rd)
        }
        _ if ALU.contains(&m) => {
            let index = ALU.iter().position(|a| *a == m).unwrap() as u32;
            let rd = low(op(0)?)?;
            let rs = if ops.len() == 3 {
                // UAL three-operand form with the destination repeated.
                let a = low(op(1)?)?;
                let b = low(op(2)?)?;
                if a == rd {
                    b
                } else if b == rd && (m == "mul" || m == "and" || m == "orr" || m == "eor") {
                    a
                } else {
                    return Err(format!("Thumb `{m}` needs the destination as one source"));
                }
            } else {
                low(op(1)?)?
            };
            half(0x4000 | (index << 6) | (rs << 3) | rd)
        }
        "bx" => half(0x4700 | (any(op(0)?)? << 3)),
        "swi" | "svc" => {
            let v = imm(ctx, op(0)?, 255, 1)?;
            half(0xDF00 | v)
        }
        "udf" => {
            let v = imm(ctx, op(0).unwrap_or("#0"), 255, 1)?;
            half(0xDE00 | v)
        }
        "push" | "pop" => {
            let mut list = register_list(operands)?;
            let extra = if m == "push" { 1 << 14 } else { 1 << 15 };
            let r = (list & extra != 0) as u32;
            list &= !extra;
            if list & !0xFF != 0 {
                return Err(format!(
                    "{m} accepts r0-r7 plus {}",
                    if m == "push" { "lr" } else { "pc" }
                ));
            }
            half((if m == "push" { 0xB400 } else { 0xBC00 }) | (r << 8) | list)
        }
        "ldmia" | "stmia" | "ldm" | "stm" | "ldmfd" | "stmea" => {
            let rb = low(op(0)?.trim_end_matches('!'))?;
            let list = register_list(&ops[1..].join(","))?;
            if list & !0xFF != 0 {
                return Err("Thumb ldm/stm accept r0-r7 only".into());
            }
            let load = m.starts_with("ld");
            half((if load { 0xC800 } else { 0xC000 }) | (rb << 8) | list)
        }
        "adr" => {
            let rd = low(op(0)?)?;
            let target = ctx.eval(op(1)?)?;
            let offset = target - ((here + 4) & !3);
            if offset % 4 != 0 || !(0..=1020).contains(&offset) {
                return Err("adr target must be word aligned and up to 1020 bytes ahead".into());
            }
            half(0xA000 | (rd << 8) | (offset as u32 / 4))
        }
        "b" | "bal" => {
            let target = ctx.eval(op(0)?)?;
            let offset = target - (here + 4);
            if !(-2048..=2046).contains(&offset) {
                return Err(format!("branch target is out of range ({offset} bytes)"));
            }
            half(0xE000 | (((offset >> 1) as u32) & 0x7FF))
        }
        "bl" => {
            let target = ctx.eval(op(0)?)?;
            let offset = target - (here + 4);
            if !(-(1 << 22)..(1 << 22)).contains(&offset) {
                return Err("bl target is out of range".into());
            }
            let hi = 0xF000 | (((offset >> 12) as u32) & 0x7FF);
            let lo = 0xF800 | (((offset >> 1) as u32) & 0x7FF);
            let mut v = half(hi);
            v.extend(half(lo));
            v
        }
        _ if is_conditional_branch(m) => {
            let cond = condition(&m[1..]).unwrap();
            let target = ctx.eval(op(0)?)?;
            if relaxed {
                let inverse = cond ^ 1;
                let offset = target - (here + 2 + 4);
                if !(-2048..=2046).contains(&offset) {
                    return Err(format!("branch target is out of range ({offset} bytes)"));
                }
                let mut v = half(0xD000 | (inverse << 8));
                v.extend(half(0xE000 | (((offset >> 1) as u32) & 0x7FF)));
                v
            } else {
                let offset = target - (here + 4);
                if !(-256..=254).contains(&offset) {
                    return Err(format!(
                        "conditional branch is out of range ({offset} bytes)"
                    ));
                }
                half(0xD000 | (cond << 8) | (((offset >> 1) as u32) & 0xFF))
            }
        }
        _ => return transfer(m, &ops, ctx),
    };
    Ok((code, None))
}

fn sp_adjust(value: i64) -> Result<Vec<u8>, String> {
    let magnitude = value.unsigned_abs();
    if !magnitude.is_multiple_of(4) || magnitude > 508 {
        return Err(format!(
            "stack adjustment {value} must be a multiple of 4 up to 508"
        ));
    }
    let sign = if value < 0 { 0x80 } else { 0 };
    Ok(half(0xB000 | sign | (magnitude as u32 / 4)))
}

fn transfer(m: &str, ops: &[String], ctx: &Context) -> Result<(Vec<u8>, Option<u32>), String> {
    let kind = match m {
        "ldr" => 0,
        "str" => 1,
        "ldrb" => 2,
        "strb" => 3,
        "ldrh" => 4,
        "strh" => 5,
        "ldsb" | "ldrsb" => 6,
        "ldsh" | "ldrsh" => 7,
        _ => return Err(format!("unknown Thumb instruction `{m}`")),
    };
    let rd = low(ops.first().ok_or("missing register")?)?;
    let address = ops.get(1).ok_or("missing address")?.trim();
    let here = ctx.here() as i64;
    if let Some(value) = address.strip_prefix('=') {
        if kind != 0 {
            return Err("`=value` is only valid with ldr".into());
        }
        let value = ctx.eval_code_address(value)? as u32;
        let slot = ctx.literal.ok_or("literal pool slot missing")? as i64;
        let offset = slot - ((here + 4) & !3);
        if !(0..=1020).contains(&offset) {
            return Err("the literal pool is too far away (Thumb reaches 1020 bytes ahead); add `.pool` after this function".into());
        }
        return Ok((half(0x4800 | (rd << 8) | (offset as u32 / 4)), Some(value)));
    }
    if !address.starts_with('[') {
        if kind != 0 {
            return Err("only ldr can load from a label".into());
        }
        let target = ctx.eval(address)?;
        let offset = target - ((here + 4) & !3);
        if offset % 4 != 0 || !(0..=1020).contains(&offset) {
            return Err("label must be word aligned and up to 1020 bytes ahead".into());
        }
        return Ok((half(0x4800 | (rd << 8) | (offset as u32 / 4)), None));
    }
    let inside = address
        .strip_prefix('[')
        .and_then(|a| a.strip_suffix(']'))
        .ok_or("malformed address")?;
    let parts = split_operands(inside);
    let rb = any(&parts[0])?;
    let offset = parts.get(1).map(String::as_str).unwrap_or("#0");
    if let Some(ro) = register(offset) {
        let rb = low(&parts[0])?;
        if ro > 7 {
            return Err("offset register must be r0-r7".into());
        }
        let opcode = [
            0x5800, 0x5000, 0x5C00, 0x5400, 0x5A00, 0x5200, 0x5600, 0x5E00,
        ][kind];
        return Ok((half(opcode | (ro << 6) | (rb << 3) | rd), None));
    }
    if rb == 15 || rb == 13 {
        let v = imm(ctx, offset, 1020, 4)?;
        let opcode = match (rb, kind) {
            (15, 0) => 0x4800,
            (13, 0) => 0x9800,
            (13, 1) => 0x9000,
            _ => return Err("only word ldr/str can address pc or sp".into()),
        };
        return Ok((half(opcode | (rd << 8) | v), None));
    }
    let rb = low(&parts[0])?;
    let (opcode, max, scale) = match kind {
        0 => (0x6800, 124, 4),
        1 => (0x6000, 124, 4),
        2 => (0x7800, 31, 1),
        3 => (0x7000, 31, 1),
        4 => (0x8800, 62, 2),
        5 => (0x8000, 62, 2),
        _ => return Err("signed loads need a register offset in Thumb code".into()),
    };
    let v = imm(ctx, offset, max, scale)?;
    Ok((half(opcode | (v << 6) | (rb << 3) | rd), None))
}
