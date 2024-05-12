// as per continued fraction expansion
use crate::Options;
use rug::{float::Constant::Pi, ops::Pow, Float, Integer};
pub fn fraction(value: Float, options: Options) -> String
{
    if value.clone().fract().is_zero() || !value.is_finite()
    {
        return String::new();
    }
    let e = Float::with_val(options.prec, 1.0).exp();
    let values = [
        None,
        None,
        None,
        Some(Float::with_val(options.prec, Pi)),
        Some(e.clone()),
        Some(1 / e),
    ];
    let sign: String = if value < 0.0
    {
        '-'.to_string()
    }
    else
    {
        String::new()
    };
    let val = value.abs();
    for (i, constant) in values.iter().enumerate()
    {
        let orig = if i == 0
        {
            val.clone()
        }
        else if i == 1
        {
            val.clone().pow(2)
        }
        else if i == 2
        {
            val.clone().pow(3)
        }
        else
        {
            val.clone() / constant.clone().unwrap()
        };
        if orig.clone().fract().is_zero()
        {
            return if i == 0
            {
                String::new()
            }
            else if i == 1 || i == 2
            {
                format!(
                    "{}{}({})",
                    sign,
                    if i == 1 { "sqrt" } else { "cbrt" },
                    if orig == 1.0
                    {
                        String::new()
                    }
                    else
                    {
                        orig.to_integer().unwrap().to_string()
                    }
                )
            }
            else
            {
                format!(
                    "{}{}{}",
                    sign,
                    if orig == 1.0
                    {
                        String::new()
                    }
                    else
                    {
                        orig.to_integer().unwrap().to_string()
                    },
                    match i
                    {
                        3 => "π",
                        4 => "e",
                        5 => "1/e",
                        _ => "",
                    }
                )
            };
        }
        let mut number = orig.clone().fract();
        let mut nums = Vec::new();
        for _ in 0..32
        {
            let mut recip = number.clone().recip();
            let fract = recip.clone().fract();
            if fract < 1e-32
            {
                let mut last = Float::with_val(options.prec, 1);
                for j in (0..nums.len()).rev()
                {
                    last.clone_from(&recip);
                    recip *= &nums[j];
                }
                let recip = match recip.to_integer()
                {
                    Some(n) => n,
                    None => return String::new(),
                };
                let last = (last + recip.clone() * orig.trunc()).to_integer().unwrap();
                return if (recip == 1 && i == 0)
                    || recip.to_string().len() > options.decimal_places
                    || last.to_string().len() > options.decimal_places
                {
                    String::new()
                }
                else if i == 1 || i == 2
                {
                    format!(
                        "{sign}{}({}{}",
                        if i == 1 { "sqrt" } else { "cbrt" },
                        if last == 1 && i != 0
                        {
                            String::new()
                        }
                        else
                        {
                            last.to_string()
                        },
                        if recip == 1
                        {
                            String::new()
                        }
                        else if i == 1
                        {
                            let (root, rem) = recip.clone().sqrt_rem(Integer::new());
                            if rem == 0
                            {
                                ")/".to_owned() + &root.to_string()
                            }
                            else
                            {
                                "/".to_owned() + &recip.to_string() + ")"
                            }
                        }
                        else
                        {
                            let (root, rem) = recip.clone().root_rem(Integer::new(), 3);
                            if rem == 0
                            {
                                ")/".to_owned() + &root.to_string()
                            }
                            else
                            {
                                "/".to_owned() + &recip.to_string() + ")"
                            }
                        }
                    )
                }
                else
                {
                    format!(
                        "{sign}{}{}{}",
                        if last == 1 && i != 0
                        {
                            String::new()
                        }
                        else
                        {
                            last.to_string()
                        },
                        match i
                        {
                            0 => String::new(),
                            3 => "π".to_string(),
                            4 => "e".to_string(),
                            5 => last.to_string(),
                            _ => String::new(),
                        },
                        if i == 5
                        {
                            if recip == 1
                            {
                                "/".to_owned() + "e"
                            }
                            else
                            {
                                "/".to_owned() + "(" + &recip.to_string() + "e" + ")"
                            }
                        }
                        else if recip == 1
                        {
                            String::new()
                        }
                        else
                        {
                            "/".to_owned() + &recip.to_string()
                        }
                    )
                };
            }
            nums.push(recip);
            number = fract;
        }
    }
    String::new()
}
