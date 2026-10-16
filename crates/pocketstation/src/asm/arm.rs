//! ARM (32-bit) instruction encoding.

use super::{Context, condition, immediate, register, register_list, split_operands};

const DATA_OPS: &[&str] = &[
    "and", "eor", "sub", "rsb", "add", "adc", "sbc", "rsc", "tst", "teq", "cmp", "cmn", "orr",
    "mov", "bic", "mvn",
];

/// Splits `mnemonic` into (base, condition, flags-suffix) given known bases.
fn split<'a>(
    mnemonic: &'a str,
    bases: &[&'a str],
    suffixes: &[&'a str],
) -> Option<(&'a str, u32, &'a str)> {
    let mut best: Option<(&str, u32, &str)> = None;
    for base in bases {
        let Some(rest) = mnemonic.strip_prefix(base) else {
            continue;
        };
        for suffix in suffixes {
            // UAL: base + suffix + cond ; pre-UAL: base + cond + suffix.
            let forms = [
                rest.strip_prefix(suffix).map(|cond| (cond, *suffix)),
                rest.strip_suffix(suffix).map(|cond| (cond, *suffix)),
            ];
            for (cond, suffix) in forms.into_iter().flatten() {
                if let Some(c) = condition(cond)
                    && best.is_none_or(|(b, _, _)| base.len() > b.len())
                {
                    best = Some((base, c, suffix));
                }
            }
        }
    }
    best
}

/// Encodes a value as an ARM rotated 8-bit immediate.
pub fn rotated_immediate(value: u32) -> Option<u32> {
    for rotate in 0..16u32 {
        let v = value.rotate_left(rotate * 2);
        if v <= 0xFF {
            return Some((rotate << 8) | v);
        }
    }
    None
}

fn reg(text: &str) -> Result<u32, String> {
    register(text).ok_or_else(|| format!("expected a register, found `{}`", text.trim()))
}

fn shift_kind(text: &str) -> Option<u32> {
    match text {
        "lsl" | "asl" => Some(0),
        "lsr" => Some(1),
        "asr" => Some(2),
        "ror" => Some(3),
        _ => None,
    }
}

/// Encodes a shifter operand `rm[, shift]` (register form, bits 0-11).
fn shifted_register(
    rm: &str,
    shift: Option<&str>,
    ctx: &Context,
    allow_register_shift: bool,
) -> Result<u32, String> {
    let rm = reg(rm)?;
    let Some(shift) = shift else { return Ok(rm) };
    let shift = shift.trim().to_ascii_lowercase();
    if shift == "rrx" {
        return Ok(rm | (3 << 5));
    }
    let (kind, amount) = shift
        .split_once(char::is_whitespace)
        .ok_or_else(|| format!("malformed shift `{shift}`"))?;
    let kind = shift_kind(kind).ok_or_else(|| format!("unknown shift `{kind}`"))?;
    let amount = amount.trim();
    if let Some(rs) = register(amount) {
        if !allow_register_shift {
            return Err("register-specified shifts are not allowed here".into());
        }
        return Ok(rm | (kind << 5) | (1 << 4) | (rs << 8));
    }
    let n = ctx.eval(immediate(amount).ok_or_else(|| format!("bad shift amount `{amount}`"))?)?;
    let encoded = match (kind, n) {
        (_, 0) => return Ok(rm),
        (0, 1..=31) => n as u32,
        (1 | 2, 1..=31) => n as u32,
        (1 | 2, 32) => 0,
        (3, 1..=31) => n as u32,
        _ => return Err(format!("shift amount {n} is out of range")),
    };
    Ok(rm | (kind << 5) | (encoded << 7))
}

pub fn encode(mnemonic: &str, operands: &str, ctx: &Context) -> Result<(u32, Option<u32>), String> {
    let ops = split_operands(operands);
    // Pseudo-instructions first.
    if mnemonic == "nop" {
        return Ok((0xE1A0_0000, None));
    }
    if let Some((_, cond, _)) = split(mnemonic, &["udf"], &[""]) {
        let n = ctx.eval(
            immediate(ops.first().map(String::as_str).unwrap_or("#0")).ok_or("udf needs #imm")?,
        )? as u32;
        if n > 0xFFFF {
            return Err("udf immediate must fit in 16 bits".into());
        }
        let _ = cond;
        return Ok((0xE7F0_00F0 | ((n >> 4) << 8) | (n & 0xF), None));
    }
    if let Some((base, cond, _)) = split(mnemonic, &["push", "pop"], &[""]) {
        let list = register_list(operands)?;
        let word = if base == "push" {
            0x092D_0000 | list
        } else {
            0x08BD_0000 | list
        };
        return Ok(((cond << 28) | word, None));
    }
    if let Some((_, cond, _)) = split(mnemonic, &["adr"], &[""]) {
        let rd = reg(&ops[0])?;
        let target = ctx.eval(&ops[1])?;
        let offset = target - (ctx.here() as i64 + 8);
        let (op, magnitude) = if offset >= 0 {
            (0x4, offset)
        } else {
            (0x2, -offset)
        };
        let imm = rotated_immediate(magnitude as u32)
            .ok_or("adr target is too far for one instruction")?;
        return Ok((
            (cond << 28) | (1 << 25) | (op << 21) | (15 << 16) | (rd << 12) | imm,
            None,
        ));
    }
    if let Some((_, cond, _)) = split(mnemonic, &["bx"], &[""]) {
        return Ok(((cond << 28) | 0x012F_FF10 | reg(&ops[0])?, None));
    }
    if let Some((base, cond, _)) = split(mnemonic, &["bl", "b"], &[""]) {
        let link = base == "bl";
        let target = ctx.eval(&ops[0])?;
        let offset = target - (ctx.here() as i64 + 8);
        if offset % 4 != 0 && ctx.final_pass {
            return Err("branch target is not word aligned".into());
        }
        if !(-(1 << 25)..(1 << 25)).contains(&offset) && ctx.final_pass {
            return Err("branch target is out of range".into());
        }
        let word = (cond << 28)
            | (0b101 << 25)
            | ((link as u32) << 24)
            | (((offset >> 2) as u32) & 0xFF_FFFF);
        return Ok((word, None));
    }
    if let Some((_, cond, _)) = split(mnemonic, &["swi", "svc"], &[""]) {
        let n = ctx.eval(immediate(&ops[0]).unwrap_or(&ops[0]))? as u32;
        return Ok(((cond << 28) | 0x0F00_0000 | (n & 0xFF_FFFF), None));
    }
    if let Some((_, cond, _)) = split(mnemonic, &["mrs"], &[""]) {
        let rd = reg(&ops[0])?;
        let spsr = ops[1].trim().to_ascii_lowercase().starts_with("spsr");
        return Ok((
            (cond << 28) | 0x010F_0000 | ((spsr as u32) << 22) | (rd << 12),
            None,
        ));
    }
    if let Some((_, cond, _)) = split(mnemonic, &["msr"], &[""]) {
        let target = ops[0].trim().to_ascii_lowercase();
        let (psr, fields) = target.split_once('_').unwrap_or((&target, "fc"));
        let spsr = psr == "spsr";
        let mut mask = 0;
        let fields = if fields == "all" {
            "fsxc"
        } else if fields == "ctl" {
            "c"
        } else if fields == "flg" {
            "f"
        } else {
            fields
        };
        for f in fields.chars() {
            mask |= match f {
                'c' => 1,
                'x' => 2,
                's' => 4,
                'f' => 8,
                _ => return Err(format!("unknown PSR field `{f}`")),
            };
        }
        let base = (cond << 28) | 0x0120_F000 | ((spsr as u32) << 22) | (mask << 16);
        if let Some(rm) = register(&ops[1]) {
            return Ok((base | rm, None));
        }
        let value = ctx.eval(immediate(&ops[1]).ok_or("msr needs a register or #imm")?)? as u32;
        let imm = rotated_immediate(value).ok_or("msr immediate is not encodable")?;
        return Ok((base | (1 << 25) | imm, None));
    }
    if let Some((base, cond, flags)) = split(mnemonic, &["mul", "mla"], &["s", ""]) {
        let set = (flags == "s") as u32;
        let rd = reg(&ops[0])?;
        let rm = reg(&ops[1])?;
        let rs = reg(&ops[2])?;
        let mut word = (cond << 28) | (set << 20) | (rd << 16) | (rs << 8) | 0x90 | rm;
        if base == "mla" {
            word |= (1 << 21) | (reg(&ops[3])? << 12);
        }
        return Ok((word, None));
    }
    if let Some((base, cond, flags)) =
        split(mnemonic, &["umull", "umlal", "smull", "smlal"], &["s", ""])
    {
        let set = (flags == "s") as u32;
        let lo = reg(&ops[0])?;
        let hi = reg(&ops[1])?;
        let rm = reg(&ops[2])?;
        let rs = reg(&ops[3])?;
        let signed = base.starts_with('s') as u32;
        let acc = base.ends_with("lal") as u32;
        let word = (cond << 28)
            | 0x0080_0090
            | (signed << 22)
            | (acc << 21)
            | (set << 20)
            | (hi << 16)
            | (lo << 12)
            | (rs << 8)
            | rm;
        return Ok((word, None));
    }
    if let Some((_, cond, flags)) = split(mnemonic, &["swp"], &["b", ""]) {
        let byte = (flags == "b") as u32;
        let rd = reg(&ops[0])?;
        let rm = reg(&ops[1])?;
        let rn = reg(ops[2].trim().trim_start_matches('[').trim_end_matches(']'))?;
        return Ok((
            (cond << 28) | 0x0100_0090 | (byte << 22) | (rn << 16) | (rd << 12) | rm,
            None,
        ));
    }
    if let Some((base, cond, mode)) = split(
        mnemonic,
        &["ldm", "stm"],
        &["ia", "ib", "da", "db", "fd", "ed", "fa", "ea", ""],
    ) {
        let load = base == "ldm";
        let (pre, up) = match (mode, load) {
            ("ia", _) | ("", _) => (false, true),
            ("ib", _) => (true, true),
            ("da", _) => (false, false),
            ("db", _) => (true, false),
            ("fd", true) | ("ed", false) => (false, true),
            ("fd", false) | ("ed", true) => (true, false),
            ("fa", true) | ("ea", false) => (false, false),
            ("fa", false) | ("ea", true) => (true, true),
            _ => return Err(format!("unknown addressing mode `{mode}`")),
        };
        let base_text = ops[0].trim();
        let writeback = base_text.ends_with('!');
        let rn = reg(base_text.trim_end_matches('!'))?;
        let list_text = ops[1..].join(",");
        let user = list_text.trim().ends_with('^');
        let list = register_list(list_text.trim().trim_end_matches('^'))?;
        let word = (cond << 28)
            | (0b100 << 25)
            | ((pre as u32) << 24)
            | ((up as u32) << 23)
            | ((user as u32) << 22)
            | ((writeback as u32) << 21)
            | ((load as u32) << 20)
            | (rn << 16)
            | list;
        return Ok((word, None));
    }
    if let Some((base, cond, kind)) = split(
        mnemonic,
        &["ldr", "str"],
        &["sb", "sh", "b", "h", "t", "bt", ""],
    ) {
        return memory(base == "ldr", cond, kind, &ops, ctx);
    }
    if let Some((base, cond, flags)) = split(mnemonic, DATA_OPS, &["s", ""]) {
        return data(base, cond, flags == "s", &ops, ctx).map(|w| (w, None));
    }
    // Shift pseudo-instructions: lsl rd, rm, #n == mov rd, rm, lsl #n
    if let Some((base, cond, flags)) = split(mnemonic, &["lsl", "lsr", "asr", "ror"], &["s", ""]) {
        let rd = &ops[0];
        let (rm, amount) = if ops.len() == 3 {
            (&ops[1], &ops[2])
        } else {
            (&ops[0], &ops[1])
        };
        let shift = format!("{base} {amount}");
        let operand2 = shifted_register(rm, Some(&shift), ctx, true)?;
        let word = (cond << 28)
            | (0xD << 21)
            | (((flags == "s") as u32) << 20)
            | (reg(rd)? << 12)
            | operand2;
        return Ok((word, None));
    }
    Err(format!("unknown ARM instruction `{mnemonic}`"))
}

fn data(base: &str, cond: u32, set: bool, ops: &[String], ctx: &Context) -> Result<u32, String> {
    let mut opcode = DATA_OPS.iter().position(|o| *o == base).unwrap() as u32;
    let compare = (8..=11).contains(&opcode);
    let single = opcode == 13 || opcode == 15;
    let mut set = set || compare;
    let (rd, rn, rest) = if compare {
        (0, reg(&ops[0])?, &ops[1..])
    } else if single {
        (reg(&ops[0])?, 0, &ops[1..])
    } else if ops.len() >= 3 {
        (reg(&ops[0])?, reg(&ops[1])?, &ops[2..])
    } else {
        // Two-operand shorthand: add r0, #1 == add r0, r0, #1
        let rd = reg(&ops[0])?;
        (rd, rd, &ops[1..])
    };
    if rest.is_empty() {
        return Err(format!("`{base}` needs an operand"));
    }
    let operand2;
    if let Some(imm) = immediate(&rest[0]).filter(|_| register(&rest[0]).is_none()) {
        let value = ctx.eval(imm)? as u32;
        let encoded = rotated_immediate(value);
        operand2 = match encoded {
            Some(e) => (1 << 25) | e,
            None => {
                // Try the complementary instruction.
                let (alt, alt_value) = match opcode {
                    13 => (15, !value),
                    15 => (13, !value),
                    4 => (2, value.wrapping_neg()),
                    2 => (4, value.wrapping_neg()),
                    0 => (14, !value),
                    14 => (0, !value),
                    10 => (11, value.wrapping_neg()),
                    11 => (10, value.wrapping_neg()),
                    5 => (6, !value),
                    6 => (5, !value),
                    _ => return Err(format!("immediate {value:#x} cannot be encoded")),
                };
                let e = rotated_immediate(alt_value).ok_or_else(|| {
                    if ctx.final_pass {
                        format!("immediate {value:#x} cannot be encoded; use `ldr rX, ={value:#x}`")
                    } else {
                        String::new()
                    }
                });
                let e = match e {
                    Ok(e) => e,
                    Err(m) if m.is_empty() => 0,
                    Err(m) => return Err(m),
                };
                opcode = alt;
                (1 << 25) | e
            }
        };
    } else {
        operand2 = shifted_register(&rest[0], rest.get(1).map(String::as_str), ctx, true)?;
    }
    if compare {
        set = true;
    }
    Ok((cond << 28) | (opcode << 21) | ((set as u32) << 20) | (rn << 16) | (rd << 12) | operand2)
}

fn memory(
    load: bool,
    cond: u32,
    kind: &str,
    ops: &[String],
    ctx: &Context,
) -> Result<(u32, Option<u32>), String> {
    let rd = reg(&ops[0])?;
    let address = ops.get(1).ok_or("missing address operand")?.trim();
    let halfword = matches!(kind, "h" | "sh" | "sb");
    if halfword && !load && kind != "h" {
        return Err("signed stores do not exist".into());
    }
    // Literal: ldr rd, =value
    if let Some(value) = address.strip_prefix('=') {
        if !load || !kind.is_empty() {
            return Err("`=value` is only valid with ldr".into());
        }
        let value = ctx.eval_code_address(value)? as u32;
        let slot = ctx.literal.ok_or("literal pool slot missing")?;
        let offset = slot as i64 - (ctx.here() as i64 + 8);
        if !(-4095..=4095).contains(&offset) {
            return Err("literal pool is too far; add a `.pool` directive nearby".into());
        }
        let up = (offset >= 0) as u32;
        let word = (cond << 28)
            | (0b01 << 26)
            | (1 << 24)
            | (up << 23)
            | (1 << 20)
            | (15 << 16)
            | (rd << 12)
            | offset.unsigned_abs() as u32;
        return Ok((word, Some(value)));
    }
    let (rn, offset_text, pre, writeback) = if address.starts_with('[') {
        let close = address.find(']').ok_or("missing `]`")?;
        let inside = &address[1..close];
        let after = address[close + 1..].trim();
        let parts = split_operands(inside);
        let rn = reg(&parts[0])?;
        if after.starts_with('!') {
            (rn, parts[1..].to_vec(), true, true)
        } else if after.is_empty() && ops.len() == 2 {
            (rn, parts[1..].to_vec(), true, false)
        } else {
            // Post-indexed: [rn], offset
            (rn, ops[2..].to_vec(), false, false)
        }
    } else {
        // PC-relative label.
        let target = ctx.eval(address)?;
        let offset = target - (ctx.here() as i64 + 8);
        (15, vec![format!("#{offset}")], true, false)
    };
    let mut up = true;
    let mut register_offset = false;
    let mut offset_bits = 0u32;
    if let Some(first) = offset_text.first() {
        let mut first = first.trim().to_string();
        if let Some(rest) = first.strip_prefix('-').filter(|r| register(r).is_some()) {
            up = false;
            first = rest.to_string();
        } else if let Some(rest) = first.strip_prefix('+') {
            first = rest.to_string();
        }
        if register(&first).is_some() {
            register_offset = true;
            if halfword {
                if offset_text.len() > 1 {
                    return Err("halfword transfers cannot shift the offset register".into());
                }
                offset_bits = reg(&first)?;
            } else {
                offset_bits =
                    shifted_register(&first, offset_text.get(1).map(String::as_str), ctx, false)?;
            }
        } else {
            let value =
                ctx.eval(immediate(&first).ok_or_else(|| format!("bad offset `{first}`"))?)?;
            if value < 0 {
                up = false;
            }
            let magnitude = value.unsigned_abs() as u32;
            let limit = if halfword { 255 } else { 4095 };
            if magnitude > limit {
                return Err(format!("offset {value} is out of range"));
            }
            offset_bits = if halfword {
                ((magnitude & 0xF0) << 4) | (magnitude & 0xF)
            } else {
                magnitude
            };
        }
    }
    if halfword {
        let sh = match kind {
            "h" => 1,
            "sb" => 2,
            _ => 3,
        };
        let word = (cond << 28)
            | ((pre as u32) << 24)
            | ((up as u32) << 23)
            | ((!register_offset as u32) << 22)
            | ((writeback as u32) << 21)
            | ((load as u32) << 20)
            | (rn << 16)
            | (rd << 12)
            | 0x90
            | (sh << 5)
            | offset_bits;
        return Ok((word, None));
    }
    let byte = kind.starts_with('b') as u32;
    let translate = kind.ends_with('t');
    let word = (cond << 28)
        | (0b01 << 26)
        | ((register_offset as u32) << 25)
        | ((pre as u32) << 24)
        | ((up as u32) << 23)
        | (byte << 22)
        | (((writeback || (translate && !pre)) as u32) << 21)
        | ((load as u32) << 20)
        | (rn << 16)
        | (rd << 12)
        | offset_bits;
    Ok((word, None))
}
