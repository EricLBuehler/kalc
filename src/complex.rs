use crate::{
    complex::NumStr::{Matrix, Num, Str, Vector},
    math::do_math,
    Options,
};
use rug::{
    float::{
        Constant::Pi,
        Special::{Infinity, Nan},
    },
    ops::Pow,
    Complex, Float, Integer,
};
use std::{
    cmp::Ordering,
    ops::{Shl, Shr},
};
#[derive(Clone, PartialEq)]
pub enum NumStr
{
    Num(Complex),
    Str(String),
    Vector(Vec<Complex>),
    Matrix(Vec<Vec<Complex>>),
}
impl NumStr
{
    pub fn mul(&self, b: &Self) -> Result<Self, &'static str>
    {
        fn m(a: &Complex, b: &Complex) -> Complex
        {
            if a.real().is_infinite() || b.real().is_infinite()
            {
                if (a.real().is_infinite() && b.is_zero())
                    || (b.real().is_infinite() && a.is_zero())
                {
                    Complex::with_val(a.prec(), Nan)
                }
                else
                {
                    match (a.real().is_sign_positive(), b.real().is_sign_positive())
                    {
                        (true, true) | (false, false) => Complex::with_val(a.prec(), Infinity),
                        (false, true) | (true, false) => -Complex::with_val(a.prec(), Infinity),
                    }
                }
            }
            else
            {
                a * b.clone()
            }
        }
        Ok(match (self, b)
        {
            (Num(a), Num(b)) => Num(m(a, b)),
            (Num(b), Vector(a)) | (Vector(a), Num(b)) =>
            {
                Vector(a.iter().map(|a| a * b.clone()).collect())
            }
            (Vector(a), Vector(b)) if a.len() == b.len() =>
            {
                Vector(a.iter().zip(b.iter()).map(|(a, b)| a * b.clone()).collect())
            }
            (Num(b), Matrix(a)) | (Matrix(a), Num(b)) => Matrix(
                a.iter()
                    .map(|a| a.iter().map(|a| a * b.clone()).collect())
                    .collect(),
            ),
            (Vector(b), Matrix(a)) if a[0].len() == b.len() => Vector(
                a.iter()
                    .map(|a| {
                        a.iter()
                            .zip(b.iter())
                            .map(|(a, b)| a * b.clone())
                            .fold(Complex::new(b[0].prec()), |sum, val| sum + val)
                    })
                    .collect::<Vec<Complex>>(),
            ),
            (Matrix(a), Vector(b)) if a[0].len() == b.len() => Matrix(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a.iter().map(|a| a * b.clone()).collect::<Vec<Complex>>())
                    .collect::<Vec<Vec<Complex>>>(),
            ),
            (Matrix(a), Matrix(b))
                if a.len() == b[0].len() && (0..b.len()).all(|j| b.len() == b[j].len()) =>
            {
                Matrix(
                    a.iter()
                        .map(|a| {
                            transpose(b)
                                .unwrap()
                                .iter()
                                .map(|b| {
                                    a.iter()
                                        .zip(b.iter())
                                        .map(|(a, b)| a * b.clone())
                                        .fold(Complex::new(a[0].prec()), |sum, val| sum + val)
                                })
                                .collect::<Vec<Complex>>()
                        })
                        .collect(),
                )
            }
            _ => return Err("mul err"),
        })
    }
    pub fn pm(&self, b: &Self) -> Result<Self, &'static str>
    {
        Ok(match (self, b)
        {
            (Num(a), Num(b)) => Vector(vec![a + b.clone(), a - b.clone()]),
            (Num(a), Vector(b)) => Vector(
                b.iter()
                    .map(|b| a + b.clone())
                    .chain(b.iter().map(|b| a - b.clone()))
                    .collect(),
            ),
            (Vector(b), Num(a)) => Vector(
                b.iter()
                    .map(|b| b + a.clone())
                    .chain(b.iter().map(|b| b - a.clone()))
                    .collect(),
            ),
            (Vector(a), Vector(b)) if a.len() == b.len() => Vector(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a + b.clone())
                    .chain(a.iter().zip(b.iter()).map(|(a, b)| a - b.clone()))
                    .collect(),
            ),
            (Matrix(a), Num(b)) | (Num(b), Matrix(a)) => Vector(
                a.iter()
                    .flat_map(|a| {
                        a.iter()
                            .map(|a| a + b.clone())
                            .chain(a.iter().map(|a| a - b.clone()))
                            .collect::<Vec<Complex>>()
                    })
                    .collect::<Vec<Complex>>(),
            ),
            _ => return Err("plus-minus unsupported"),
        })
    }
    pub fn pow(&self, b: &Self) -> Result<Self, &'static str>
    {
        fn p(a: &Complex, b: &Complex) -> Complex
        {
            if a.real().is_infinite()
            {
                if b.is_zero()
                {
                    Complex::with_val(a.prec(), Nan)
                }
                else if b.real().is_sign_negative()
                {
                    Complex::with_val(a.prec(), Infinity)
                }
                else
                {
                    Complex::new(a.prec())
                }
            }
            else if b.real().is_infinite()
            {
                if a.clone().abs() == 1
                {
                    Complex::with_val(a.prec(), Nan)
                }
                else if b.real().is_sign_positive() == a.real().clone().trunc().is_zero()
                {
                    Complex::new(a.prec())
                }
                else
                {
                    Complex::with_val(a.prec(), Infinity)
                }
            }
            else if a.is_zero() && b.real().is_zero()
            {
                Complex::with_val(a.prec(), Nan)
            }
            else
            {
                a.pow(b.clone())
            }
        }
        Ok(match (self, b)
        {
            (Num(a), Num(b)) => Num(p(a, b)),
            (Num(a), Vector(b)) => Vector(b.iter().map(|b| p(a, b)).collect()),
            (Vector(a), Num(b)) => Vector(a.iter().map(|a| p(a, b)).collect()),
            (Vector(a), Vector(b)) if a.len() == b.len() =>
            {
                Vector(a.iter().zip(b.iter()).map(|(a, b)| p(a, b)).collect())
            }
            (Num(a), Matrix(b)) => Matrix(
                b.iter()
                    .map(|b| b.iter().map(|b| p(a, b)).collect())
                    .collect(),
            ),
            (Matrix(a), Num(b)) if a.len() == a[0].len() =>
            {
                if b.imag().is_zero() && b.real().clone().fract().is_zero()
                {
                    if b.real().is_zero()
                    {
                        let mut mat = Vec::new();
                        for i in 0..a.len()
                        {
                            let mut vec = Vec::new();
                            for j in 0..a.len()
                            {
                                vec.push(
                                    if i == j
                                    {
                                        Complex::with_val(a[0][0].prec(), 1)
                                    }
                                    else
                                    {
                                        Complex::new(a[0][0].prec())
                                    },
                                )
                            }
                            mat.push(vec);
                        }
                        Matrix(mat)
                    }
                    else
                    {
                        let mut mat = Matrix(a.clone());
                        let c = b.real().to_f64().abs() as usize;
                        for _ in 1..c
                        {
                            mat = mat.mul(&Matrix(a.clone()))?;
                        }
                        if b.real().is_sign_positive()
                        {
                            mat
                        }
                        else
                        {
                            Matrix(inverse(&mat.mat()?)?)
                        }
                    }
                }
                else
                {
                    return Err("no imag/fractional support for powers");
                }
            }
            (Vector(b), Matrix(a)) if b.len() == a.len() => Matrix(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a.iter().map(|a| p(b, a)).collect())
                    .collect(),
            ),
            (Matrix(a), Vector(b)) if a.len() == b.len() => Matrix(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a.iter().map(|a| p(a, b)).collect())
                    .collect(),
            ),
            (Matrix(a), Matrix(b)) if a.len() == b[0].len() && a.len() == b.len() => Matrix(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| {
                        a.iter()
                            .zip(b.iter())
                            .map(|(a, b)| p(a, b))
                            .collect::<Vec<Complex>>()
                    })
                    .collect(),
            ),
            _ => return Err("pow err"),
        })
    }
    pub fn func<F>(&self, b: &Self, func: F) -> Result<Self, &'static str>
    where
        F: Fn(&Complex, &Complex) -> Complex,
    {
        Ok(match (self, b)
        {
            (Num(a), Num(b)) => Num(func(a, b)),
            (Num(a), Vector(b)) => Vector(b.iter().map(|b| func(a, b)).collect()),
            (Vector(a), Num(b)) => Vector(a.iter().map(|a| func(a, b)).collect()),
            (Num(a), Matrix(b)) => Matrix(
                b.iter()
                    .map(|b| b.iter().map(|b| func(a, b)).collect())
                    .collect(),
            ),
            (Matrix(a), Num(b)) => Matrix(
                a.iter()
                    .map(|a| a.iter().map(|a| func(a, b)).collect())
                    .collect(),
            ),
            (Vector(a), Vector(b)) if a.len() == b.len() =>
            {
                Vector(a.iter().zip(b.iter()).map(|(a, b)| func(a, b)).collect())
            }
            (Vector(b), Matrix(a)) if b.len() == a.len() => Matrix(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a.iter().map(|a| rem(b, a)).collect())
                    .collect(),
            ),
            (Matrix(a), Vector(b)) if a.len() == b.len() => Matrix(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| a.iter().map(|a| func(a, b)).collect())
                    .collect(),
            ),
            (Matrix(a), Matrix(b)) if a.len() == b[0].len() && a.len() == b.len() => Matrix(
                a.iter()
                    .zip(b.iter())
                    .map(|(a, b)| {
                        a.iter()
                            .zip(b.iter())
                            .map(|(a, b)| func(a, b))
                            .collect::<Vec<Complex>>()
                    })
                    .collect(),
            ),
            _ => return Err("operation err"),
        })
    }
    pub fn str_is(&self, s: &str) -> bool
    {
        match self
        {
            Str(s2) => s == s2,
            _ => false,
        }
    }
    pub fn num(&self) -> Result<Complex, &'static str>
    {
        match self
        {
            Num(n) => Ok(n.clone()),
            _ => Err("failed to get number"),
        }
    }
    pub fn vec(&self) -> Result<Vec<Complex>, &'static str>
    {
        match self
        {
            Vector(v) => Ok(v.clone()),
            _ => Err("failed to get vector"),
        }
    }
    pub fn mat(&self) -> Result<Vec<Vec<Complex>>, &'static str>
    {
        match self
        {
            Matrix(m) => Ok(m.clone()),
            _ => Err("failed to get matrix"),
        }
    }
}
pub fn and(a: &Complex, b: &Complex) -> Complex
{
    Complex::with_val(
        a.prec(),
        (a.imag().is_zero() && b.imag().is_zero() && a.real() == &1 && b.real() == &1) as u8,
    )
}
pub fn or(a: &Complex, b: &Complex) -> Complex
{
    Complex::with_val(
        a.prec(),
        (a.imag().is_zero() && b.imag().is_zero() && (a.real() == &1 || b.real() == &1)) as u8,
    )
}
pub fn sub(a: &Complex, b: &Complex) -> Complex
{
    a - b.clone()
}
pub fn div(a: &Complex, b: &Complex) -> Complex
{
    if b.is_zero() || a.real().is_infinite()
    {
        if a.is_zero() || b.real().is_infinite()
        {
            Complex::with_val(a.prec(), Nan)
        }
        else if a.real().is_sign_positive()
        {
            Complex::with_val(a.prec(), Infinity)
        }
        else
        {
            -Complex::with_val(a.prec(), Infinity)
        }
    }
    else
    {
        a / b.clone()
    }
}
pub fn root(a: &Complex, b: &Complex) -> Complex
{
    let c: Float = b.real().clone() / 2;
    match b.imag().is_zero()
        && !c.fract().is_zero()
        && b.real().clone().fract().is_zero()
        && a.imag().is_zero()
    {
        true => Complex::with_val(
            a.prec(),
            a.real() / a.real().clone().abs()
                * a.real().clone().abs().pow(b.real().clone().recip()),
        ),
        false => a.pow(b.clone().recip()),
    }
}
pub fn add(a: &Complex, b: &Complex) -> Complex
{
    a + b.clone()
}
pub fn shl(a: &Complex, b: &Complex) -> Complex
{
    a.clone().shl(b.real().to_u32_saturating().unwrap_or(0))
}
pub fn shr(a: &Complex, b: &Complex) -> Complex
{
    a.clone().shr(b.real().to_u32_saturating().unwrap_or(0))
}
pub fn ne(a: &Complex, b: &Complex) -> Complex
{
    let c: Complex = a - b.clone();
    let int = Integer::from(10).pow(a.prec().0 / 4);
    let re: Float = c.real().clone() * int.clone();
    let re: Float = re.round() / int.clone();
    let im: Float = c.imag().clone() * int.clone();
    let im: Float = im.round() / int;
    Complex::with_val(a.prec(), (!re.is_zero() || !im.is_zero()) as u8)
}
pub fn eq(a: &Complex, b: &Complex) -> Complex
{
    let c: Complex = a - b.clone();
    let int = Integer::from(10).pow(a.prec().0 / 4);
    let re: Float = c.real().clone() * int.clone();
    let re: Float = re.round() / int.clone();
    let im: Float = c.imag().clone() * int.clone();
    let im: Float = im.round() / int;
    Complex::with_val(a.prec(), (re.is_zero() && im.is_zero()) as u8)
}
pub fn ge(a: &Complex, b: &Complex) -> Complex
{
    Complex::with_val(a.prec(), (a.real() >= b.real()) as u8)
}
pub fn gt(a: &Complex, b: &Complex) -> Complex
{
    Complex::with_val(a.prec(), (a.real() > b.real()) as u8)
}
pub fn le(a: &Complex, b: &Complex) -> Complex
{
    Complex::with_val(a.prec(), (a.real() <= b.real()) as u8)
}
pub fn lt(a: &Complex, b: &Complex) -> Complex
{
    Complex::with_val(a.prec(), (a.real() < b.real()) as u8)
}
pub fn rem(a: &Complex, b: &Complex) -> Complex
{
    let c = a / b.clone();
    let c = Complex::with_val(
        a.prec(),
        (c.real().clone().floor(), c.imag().clone().floor()),
    );
    a - b * c
}
pub fn gamma(a: &Float) -> Float
{
    if a.is_sign_negative() && a.clone().fract().is_zero()
    {
        Float::with_val(a.prec(), Infinity)
    }
    else
    {
        a.clone().gamma()
    }
}
pub fn tetration(a: &Complex, b: &Complex) -> Complex
{
    if b.real().clone().fract().is_zero() && b.real().is_sign_positive()
    {
        (0..=b.real().to_f64() as usize)
            .fold(Complex::new(b.prec()), |tetration, _| a.pow(tetration))
    }
    else if b.real().is_sign_positive()
    {
        a.pow(tetration(a, &(b.clone() - 1)))
    }
    else if b.real() <= &-1
    {
        tetration(a, &(b.clone() + 1)).ln() / a.clone().ln()
    }
    else
    {
        let a = a.clone().ln();
        1 + (2 * b.clone() * a.clone() / (1 + a.clone()))
            - (b.clone().pow(2) * (1 - a.clone()) / (1 + a))
    }
}
pub fn slog(a: &Complex, b: &Complex) -> Complex
{
    if b.real() <= &0
    {
        let z = &a.clone().pow(b);
        if z.real() <= b.real()
        {
            Complex::with_val(a.prec(), Nan)
        }
        else
        {
            slog(a, z) - 1
        }
    }
    else if b.real() > &1
    {
        let z = &(b.clone().ln() / a.clone().ln());
        if z.real() >= b.real()
        {
            Complex::with_val(a.prec(), Nan)
        }
        else
        {
            slog(a, z) + 1
        }
    }
    else
    {
        let a = a.clone().ln();
        (2 * a.clone() * b.clone() / (1 + a.clone()))
            + (b.clone().pow(2) * (1 - a.clone()) / (1 + a))
            - 1
    }
}
pub fn to_polar(mut a: Vec<Complex>, to_deg: Complex) -> Vec<Complex>
{
    if a.len() == 1
    {
        a.push(Complex::new(a[0].prec()));
    }
    if a.len() != 2 && a.len() != 3
    {
        Vec::new()
    }
    else if a.len() == 2
    {
        if a[1].is_zero()
        {
            if a[0].is_zero()
            {
                vec![Complex::new(a[0].prec()), Complex::new(a[0].prec())]
            }
            else
            {
                vec![
                    a[0].clone().abs(),
                    if a[0].real().is_sign_positive()
                    {
                        Complex::with_val(a[0].prec(), 0)
                    }
                    else
                    {
                        to_deg * Complex::with_val(a[0].prec(), Pi)
                    },
                ]
            }
        }
        else
        {
            let mut n: Complex = a[0].clone().pow(2) + a[1].clone().pow(2);
            n = n.sqrt();
            vec![
                n.clone(),
                a[1].clone() / a[1].clone().abs() * (&a[0] / n).acos() * to_deg,
            ]
        }
    }
    else if a[1].is_zero()
    {
        if a[0].is_zero()
        {
            if a[2].is_zero()
            {
                vec![
                    Complex::with_val(a[0].prec(), 0),
                    Complex::with_val(a[0].prec(), 0),
                    Complex::with_val(a[0].prec(), 0),
                ]
            }
            else
            {
                vec![
                    a[2].clone().abs(),
                    Complex::with_val(a[0].prec(), 0),
                    Complex::with_val(a[0].prec(), 0),
                ]
            }
        }
        else
        {
            let mut n: Complex = a[0].clone().pow(2) + a[1].clone().pow(2) + a[2].clone().pow(2);
            n = n.sqrt();
            vec![
                n.clone(),
                (&a[2] / n).acos() * to_deg.clone(),
                Complex::with_val(a[0].prec(), 0),
            ]
        }
    }
    else
    {
        let mut n: Complex = a[0].clone().pow(2) + a[1].clone().pow(2) + a[2].clone().pow(2);
        n = n.sqrt();
        let t: Complex = a[0].clone().pow(2) + a[1].clone().pow(2);
        vec![
            n.clone(),
            (&a[2] / n).acos() * to_deg.clone(),
            a[1].clone() / a[1].clone().abs() * (&a[0] / t.sqrt()).acos() * to_deg,
        ]
    }
}
pub fn to(a: &NumStr, b: &NumStr) -> Result<NumStr, &'static str>
{
    Ok(match (a, b)
    {
        (Num(a), Num(b)) =>
        {
            let prec = a.prec();
            let a = a.real().to_f64() as isize;
            let b = b.real().to_f64() as isize;
            let vec: Vec<Complex> = if a < b
            {
                (a..=b).map(|a| Complex::with_val(prec, a)).collect()
            }
            else
            {
                (b..=a).rev().map(|a| Complex::with_val(prec, a)).collect()
            };
            if vec.is_empty()
            {
                return Err("start range greater then end range");
            }
            Vector(vec)
        }
        (Vector(a), Num(b)) =>
        {
            let prec = b.prec();
            let b = b.real().to_f64() as isize;
            let mat: Vec<Vec<Complex>> = a
                .iter()
                .map(|a| {
                    let a = a.real().to_f64() as isize;
                    if a < b
                    {
                        (a..=b).map(|a| Complex::with_val(prec, a)).collect()
                    }
                    else
                    {
                        (b..=a).rev().map(|a| Complex::with_val(prec, a)).collect()
                    }
                })
                .collect();
            if mat.is_empty() || mat.iter().any(|vec| vec.is_empty())
            {
                return Err("start range greater then end range");
            }
            Matrix(mat)
        }
        (Num(a), Vector(b)) =>
        {
            let prec = a.prec();
            let a = a.real().to_f64() as isize;
            let mat: Vec<Vec<Complex>> = b
                .iter()
                .map(|b| {
                    let b = b.real().to_f64() as isize;
                    if a < b
                    {
                        (a..=b).map(|a| Complex::with_val(prec, a)).collect()
                    }
                    else
                    {
                        (b..=a).rev().map(|a| Complex::with_val(prec, a)).collect()
                    }
                })
                .collect();
            if mat.is_empty() || mat.iter().any(|vec| vec.is_empty())
            {
                return Err("start range greater then end range");
            }
            Matrix(mat)
        }
        _ => return Err(".. err"),
    })
}
pub fn mvec(
    function: Vec<NumStr>,
    func_vars: Vec<(String, Vec<NumStr>)>,
    var: &str,
    start: isize,
    end: isize,
    mvec: bool,
    options: Options,
) -> Result<NumStr, &'static str>
{
    let mut vec = Vec::new();
    let mut mat = Vec::new();
    if start < end
    {
        for z in start..=end
        {
            let mut func = function.clone();
            let mut func_vars = func_vars.clone();
            let mut bracket = 0;
            let mut sum: Vec<usize> = Vec::new();
            for (i, k) in func.clone().iter().enumerate()
            {
                if let Str(s) = k
                {
                    if s == var && sum.is_empty()
                    {
                        func[i] = Num(Complex::with_val(options.prec, z));
                    }
                    else
                    {
                        match s.as_str()
                        {
                            "(" => bracket += 1,
                            ")" =>
                            {
                                bracket -= 1;
                                if sum.contains(&bracket)
                                {
                                    sum.pop();
                                }
                            }
                            "sum" | "summation" | "prod" | "product" | "Σ" | "Π" | "vec"
                            | "mat" | "D" | "integrate" | "arclength" | "area" | "length"
                            | "slope"
                                if i + 2 < func.len() && func[i + 2] == Str(var.to_string()) =>
                            {
                                sum.push(bracket)
                            }
                            _ =>
                            {}
                        }
                    }
                }
            }
            for k in func_vars.iter_mut()
            {
                let mut dirty = false;
                if !k.0.contains('(')
                {
                    for f in k.1.iter_mut()
                    {
                        if f.str_is(var)
                        {
                            *f = Num(Complex::with_val(options.prec, z));
                            dirty = true
                        }
                    }
                }
                if dirty
                {
                    let num = do_math(k.1.clone(), options, Vec::new());
                    for (i, f) in func.clone().iter().enumerate()
                    {
                        if f.str_is(&k.0)
                        {
                            func.remove(i);
                            func.splice(i..i, num.clone());
                        }
                    }
                }
            }
            let math = do_math(func, options, func_vars.clone())?;
            match math
            {
                Num(n) => vec.push(n),
                Vector(v) if mvec => vec.extend(v),
                Vector(v) => mat.push(v),
                Matrix(m) if !mvec => mat.extend(m),
                _ => return Err("cant create 3d matrix"),
            }
        }
    }
    else
    {
        for z in (end..=start).rev()
        {
            let mut func = function.clone();
            let mut func_vars = func_vars.clone();
            let mut bracket = 0;
            let mut sum: Vec<usize> = Vec::new();
            for (i, k) in func.clone().iter().enumerate()
            {
                if let Str(s) = k
                {
                    if s == var && sum.is_empty()
                    {
                        func[i] = Num(Complex::with_val(options.prec, z));
                    }
                    else
                    {
                        match s.as_str()
                        {
                            "(" => bracket += 1,
                            ")" =>
                            {
                                bracket -= 1;
                                if sum.contains(&bracket)
                                {
                                    sum.pop();
                                }
                            }
                            "sum" | "summation" | "prod" | "product" | "Σ" | "Π" | "vec"
                            | "mat" | "D" | "integrate" | "arclength" | "area" | "length"
                            | "slope"
                                if i + 2 < func.len() && func[i + 2] == Str(var.to_string()) =>
                            {
                                sum.push(bracket)
                            }
                            _ =>
                            {}
                        }
                    }
                }
            }
            for k in func_vars.iter_mut()
            {
                let mut dirty = false;
                if !k.0.contains('(')
                {
                    for f in k.1.iter_mut()
                    {
                        if f.str_is(var)
                        {
                            *f = Num(Complex::with_val(options.prec, z));
                            dirty = true
                        }
                    }
                }
                if dirty
                {
                    let num = do_math(k.1.clone(), options, Vec::new());
                    for (i, f) in func.clone().iter().enumerate()
                    {
                        if f.str_is(&k.0)
                        {
                            func.remove(i);
                            func.splice(i..i, num.clone());
                        }
                    }
                }
            }
            let math = do_math(func, options, func_vars.clone())?;
            match math
            {
                Num(n) => vec.push(n),
                Vector(v) if mvec => vec.extend(v),
                Vector(v) => mat.push(v),
                Matrix(m) if !mvec => mat.extend(m),
                _ => return Err("cant create 3d matrix"),
            }
        }
    }
    if mat.is_empty()
    {
        if vec.is_empty()
        {
            Err("start>end")
        }
        else
        {
            Ok(Vector(vec))
        }
    }
    else
    {
        Ok(Matrix(mat))
    }
}
pub fn sum(
    function: Vec<NumStr>,
    func_vars: Vec<(String, Vec<NumStr>)>,
    var: &str,
    start: isize,
    end: isize,
    product: bool,
    options: Options,
) -> Result<NumStr, &'static str>
{
    let mut value = Num(
        if product
        {
            Complex::with_val(options.prec, 1)
        }
        else
        {
            Complex::new(options.prec)
        },
    );
    for z in if start < end
    {
        start..=end
    }
    else
    {
        end..=start
    }
    {
        let mut func = function.clone();
        let mut func_vars = func_vars.clone();
        let mut bracket = 0;
        let mut sum: Vec<usize> = Vec::new();
        for (i, k) in func.clone().iter().enumerate()
        {
            if let Str(s) = k
            {
                if s == var && sum.is_empty()
                {
                    func[i] = Num(Complex::with_val(options.prec, z));
                }
                else
                {
                    match s.as_str()
                    {
                        "(" => bracket += 1,
                        ")" =>
                        {
                            bracket -= 1;
                            if sum.contains(&bracket)
                            {
                                sum.pop();
                            }
                        }
                        "sum" | "summation" | "prod" | "product" | "Σ" | "Π" | "vec" | "mat"
                        | "D" | "integrate" | "arclength" | "area" | "length" | "slope"
                            if i + 2 < func.len() && func[i + 2] == Str(var.to_string()) =>
                        {
                            sum.push(bracket)
                        }
                        _ =>
                        {}
                    }
                }
            }
        }
        for k in func_vars.iter_mut()
        {
            let mut dirty = false;
            if !k.0.contains('(')
            {
                for f in k.1.iter_mut()
                {
                    if f.str_is(var)
                    {
                        *f = Num(Complex::with_val(options.prec, z));
                        dirty = true
                    }
                }
            }
            if dirty
            {
                let num = do_math(k.1.clone(), options, Vec::new());
                for (i, f) in func.clone().iter().enumerate()
                {
                    if f.str_is(&k.0)
                    {
                        func.remove(i);
                        func.splice(i..i, num.clone());
                    }
                }
            }
        }
        let math = do_math(func, options, func_vars.clone())?;
        if product
        {
            value = value.mul(&math)?;
        }
        else
        {
            value = value.func(&math, add)?;
        }
    }
    Ok(value)
}
pub fn submatrix(a: &[Vec<Complex>], row: usize, col: usize) -> Vec<Vec<Complex>>
{
    a.iter()
        .enumerate()
        .filter(|&(i, _)| i != row)
        .map(|(_, r)| {
            r.iter()
                .enumerate()
                .filter(|&(j, _)| j != col)
                .map(|(_, value)| value.clone())
                .collect::<Vec<Complex>>()
        })
        .collect()
}
pub fn trace(a: &[Vec<Complex>]) -> Complex
{
    let mut n = Complex::new(a[0][0].prec());
    for (i, j) in a.iter().enumerate()
    {
        if j.len() == i
        {
            break;
        }
        n += j[i].clone();
    }
    n
}
pub fn identity(a: usize, prec: (u32, u32)) -> Vec<Vec<Complex>>
{
    let mut mat = Vec::with_capacity(a);
    for i in 0..a
    {
        let mut vec = Vec::with_capacity(a);
        for j in 0..a
        {
            if i == j
            {
                vec.push(Complex::with_val(prec, 1));
            }
            else
            {
                vec.push(Complex::new(prec));
            }
        }
        mat.push(vec);
    }
    mat
}
pub fn determinant(a: &[Vec<Complex>]) -> Result<Complex, &'static str>
{
    if !a.is_empty() && (0..a.len()).all(|j| a.len() == a[j].len())
    {
        Ok(match a.len()
        {
            1 => a[0][0].clone(),
            2 => a[0][0].clone() * a[1][1].clone() - a[1][0].clone() * a[0][1].clone(),
            3 =>
            {
                a[0][0].clone()
                    * (a[1][1].clone() * a[2][2].clone() - a[1][2].clone() * a[2][1].clone())
                    + a[0][1].clone()
                        * (a[1][2].clone() * a[2][0].clone() - a[1][0].clone() * a[2][2].clone())
                    + a[0][2].clone()
                        * (a[1][0].clone() * a[2][1].clone() - a[1][1].clone() * a[2][0].clone())
            }
            _ =>
            {
                let mut det = Complex::new(a[0][0].prec());
                for (i, x) in a[0].iter().enumerate()
                {
                    let mut sub_matrix = a[1..].to_vec();
                    for row in &mut sub_matrix
                    {
                        row.remove(i);
                    }
                    det += x * determinant(&sub_matrix)? * if i % 2 == 0 { 1.0 } else { -1.0 };
                }
                det
            }
        })
    }
    else
    {
        Err("not square")
    }
}
pub fn transpose(a: &[Vec<Complex>]) -> Result<Vec<Vec<Complex>>, &'static str>
{
    if (0..a.len()).all(|j| a.len() == a[j].len())
    {
        let mut b = vec![vec![Complex::new(1); a.len()]; a[0].len()];
        for (i, l) in a.iter().enumerate()
        {
            for (j, n) in l.iter().enumerate()
            {
                b[j][i] = n.clone();
            }
        }
        Ok(b)
    }
    else
    {
        Err("not square")
    }
}
pub fn minors(a: &[Vec<Complex>]) -> Result<Vec<Vec<Complex>>, &'static str>
{
    if (0..a.len()).all(|j| a.len() == a[j].len())
    {
        let mut result = vec![vec![Complex::new(1); a[0].len()]; a.len()];
        for (i, k) in result.iter_mut().enumerate()
        {
            for (j, l) in k.iter_mut().enumerate()
            {
                *l = determinant(&submatrix(a, i, j))?
            }
        }
        Ok(result)
    }
    else
    {
        Err("not square")
    }
}
pub fn cofactor(a: &[Vec<Complex>]) -> Result<Vec<Vec<Complex>>, &'static str>
{
    if (0..a.len()).all(|j| a.len() == a[j].len())
    {
        let mut result = vec![vec![Complex::new(1); a[0].len()]; a.len()];
        for (i, k) in result.iter_mut().enumerate()
        {
            for (j, l) in k.iter_mut().enumerate()
            {
                *l = if (i + j) % 2 == 1
                {
                    -determinant(&submatrix(a, i, j))?
                }
                else
                {
                    determinant(&submatrix(a, i, j))?
                };
            }
        }
        Ok(result)
    }
    else
    {
        Err("not square")
    }
}
pub fn inverse(a: &[Vec<Complex>]) -> Result<Vec<Vec<Complex>>, &'static str>
{
    if (0..a.len()).all(|j| a.len() == a[j].len())
    {
        Matrix(transpose(&cofactor(a)?)?)
            .func(&Num(determinant(a)?), div)?
            .mat()
    }
    else
    {
        Err("not square")
    }
}
pub fn nth_prime(n: usize) -> usize
{
    let mut count = 0;
    let mut num = 2;
    if n == 0
    {
        num = 0
    }
    while count < n
    {
        if is_prime(num)
        {
            count += 1;
        }
        if count < n
        {
            num += 1;
        }
    }
    num
}
pub fn is_prime(num: usize) -> bool
{
    if num <= 1
    {
        return false;
    }
    if num <= 3
    {
        return true;
    }
    if num % 2 == 0 || num % 3 == 0
    {
        return false;
    }
    let mut i = 5;
    while i * i <= num
    {
        if num % i == 0 || num % (i + 2) == 0
        {
            return false;
        }
        i += 6;
    }
    true
}
pub fn sort(mut a: Vec<Complex>) -> Vec<Complex>
{
    a.sort_by(|x, y| x.real().partial_cmp(y.real()).unwrap_or(Ordering::Equal));
    a
}
pub fn eigenvalues(a: &[Vec<Complex>]) -> Result<Vec<Complex>, &'static str>
{
    if !a.is_empty() && (0..a.len()).all(|j| a.len() == a[j].len())
    {
        match a.len()
        {
            1 => Ok(a[0].clone()),
            2 => Ok(quadratic(
                Complex::with_val(a[0][0].prec(), 1),
                -a[0][0].clone() - a[1][1].clone(),
                a[0][0].clone() * a[1][1].clone() - a[0][1].clone() * a[1][0].clone(),
                false,
            )),
            3 => Ok(cubic(
                Complex::with_val(a[0][0].prec(), -1),
                a[2][2].clone() + a[1][1].clone() + a[0][0].clone(),
                -a[0][0].clone() * a[1][1].clone() - a[0][0].clone() * a[2][2].clone()
                    + a[0][1].clone() * a[1][0].clone()
                    + a[0][2].clone() * a[2][0].clone()
                    - a[1][1].clone() * a[2][2].clone()
                    + a[1][2].clone() * a[2][1].clone(),
                a[0][0].clone() * a[1][1].clone() * a[2][2].clone()
                    - a[0][0].clone() * a[1][2].clone() * a[2][1].clone()
                    - a[0][1].clone() * a[1][0].clone() * a[2][2].clone()
                    + a[0][1].clone() * a[1][2].clone() * a[2][0].clone()
                    + a[0][2].clone() * a[1][0].clone() * a[2][1].clone()
                    - a[0][2].clone() * a[1][1].clone() * a[2][0].clone(),
                false,
            )),
            _ => Err("unsupported"),
        }
    }
    else
    {
        Err("not square")
    }
}
pub fn quadratic(a: Complex, b: Complex, c: Complex, real: bool) -> Vec<Complex>
{
    if a.is_zero()
    {
        return vec![-c / b];
    }
    let p: Complex = b.clone().pow(2);
    let p: Complex = p - (4 * c * a.clone());
    let p = p.sqrt();
    let a: Complex = 2 * a;
    if real
    {
        let z1 = (p.clone() - b.clone()) / a.clone();
        let z2 = (-p - b) / a;
        let mut vec = Vec::new();
        if z1.imag().to_f64().abs() < 0.0000000000000001
        {
            vec.push(z1)
        }
        if z2.imag().to_f64().abs() < 0.0000000000000001
        {
            vec.push(z2)
        }
        vec
    }
    else
    {
        vec![(p.clone() - b.clone()) / a.clone(), (-p - b) / a]
    }
}
pub fn cubic(a: Complex, b: Complex, c: Complex, d: Complex, real: bool) -> Vec<Complex>
{
    if a.is_zero()
    {
        return quadratic(b, c, d, real);
    }
    let prec = a.prec();
    let threerecip = Float::with_val(prec.0, 3).recip();
    if b.is_zero() && c.is_zero()
    {
        return if d.is_zero()
        {
            vec![Complex::new(prec), Complex::new(prec), Complex::new(prec)]
        }
        else
        {
            let reuse = (d / a).pow(threerecip.clone());
            vec![
                -reuse.clone(),
                reuse.clone() * Complex::with_val(prec, -1).pow(threerecip.clone()),
                -reuse * Complex::with_val(prec, -1).pow(2 * threerecip),
            ]
        };
    }
    let b = b / a.clone();
    let c = c / a.clone();
    let d = d / a.clone();
    let threesqrt = Float::with_val(prec.0, 3).sqrt();
    let cbrtwo = Float::with_val(prec.0, 2).pow(threerecip.clone());
    let mut reuse: Complex = (4 * b.clone().pow(3) * d.clone())
        - (b.clone().pow(2) * c.clone().pow(2))
        - (18 * b.clone() * c.clone() * d.clone())
        + (4 * c.clone().pow(3))
        + (27 * d.clone().pow(2));
    reuse = (-2 * b.clone().pow(3))
        + (3 * threesqrt.clone() * reuse.clone().sqrt())
        + (9 * b.clone() * c.clone())
        - (27 * d.clone());
    reuse = reuse.pow(threerecip.clone());
    let left: Complex = reuse.clone() / cbrtwo.clone();
    let right: Complex = cbrtwo * (3 * c.clone() - b.clone().pow(2)) / reuse.clone();
    //(-2 b^3 + 3 sqrt(3) sqrt(4 b^3 d - b^2 c^2 - 18 b c d + 4 c^3 + 27 d^2) + 9 b c - 27 d)^(1/3)/(3 2^(1/3)) - (2^(1/3) (3 c - b^2))/(3 (-2 b^3 + 3 sqrt(3) sqrt(4 b^3 d - b^2 c^2 - 18 b c d + 4 c^3 + 27 d^2) + 9 b c - 27 d)^(1/3)) - b/3
    //-((1 - i sqrt(3)) (-2 b^3 + 3 sqrt(3) sqrt(4 b^3 d - b^2 c^2 - 18 b c d + 4 c^3 + 27 d^2) + 9 b c - 27 d)^(1/3))/(6 2^(1/3)) + ((1 + i sqrt(3)) (3 c - b^2))/(3 2^(2/3) (-2 b^3 + 3 sqrt(3) sqrt(4 b^3 d - b^2 c^2 - 18 b c d + 4 c^3 + 27 d^2) + 9 b c - 27 d)^(1/3)) - b/3
    //-((1 + i sqrt(3)) (-2 b^3 + 3 sqrt(3) sqrt(4 b^3 d - b^2 c^2 - 18 b c d + 4 c^3 + 27 d^2) + 9 b c - 27 d)^(1/3))/(6 2^(1/3)) + ((1 - i sqrt(3)) (3 c - b^2))/(3 2^(2/3) (-2 b^3 + 3 sqrt(3) sqrt(4 b^3 d - b^2 c^2 - 18 b c d + 4 c^3 + 27 d^2) + 9 b c - 27 d)^(1/3)) - b/3
    let omega: Complex = (1 + (Complex::with_val(prec, (0, 1)) * threesqrt.clone())) / 2;
    if real
    {
        let z1: Complex = (left.clone() - right.clone() - b.clone()) / 3;
        let z2: Complex =
            ((-omega.clone() * left.clone()) + (omega.clone().conj() * right.clone()) - b.clone())
                / 3;
        let z3: Complex = ((-omega.clone().conj() * left) + (omega * right) - b.clone()) / 3;
        let mut vec = Vec::new();
        if z1.imag().to_f64().abs() < 0.0000000000000001
        {
            vec.push(z1)
        }
        if z2.imag().to_f64().abs() < 0.0000000000000001
        {
            vec.push(z2)
        }
        if z3.imag().to_f64().abs() < 0.0000000000000001
        {
            vec.push(z3)
        }
        vec
    }
    else
    {
        vec![
            (left.clone() - right.clone() - b.clone()) / 3,
            ((-omega.clone() * left.clone()) + (omega.clone().conj() * right.clone()) - b.clone())
                / 3,
            ((-omega.clone().conj() * left) + (omega * right) - b.clone()) / 3,
        ]
    }
}
pub fn variance(a: &[Complex], prec: (u32, u32)) -> Complex
{
    let mean = a.iter().fold(Complex::new(prec), |sum, val| sum + val) / a.len();
    let mut variance = Complex::new(prec);
    for a in a
    {
        variance += (a - mean.clone()).pow(2)
    }
    variance / (a.len() - 1)
}
pub fn recursion(
    mut func_vars: Vec<(String, Vec<NumStr>)>,
    mut func: Vec<NumStr>,
    options: Options,
) -> Result<NumStr, &'static str>
{
    for fv in func_vars.clone()
    {
        if fv.0.ends_with(')')
        {
            if fv.0.contains(',')
            {
                let mut vars = fv.0.split(',').collect::<Vec<&str>>();
                vars[0] = vars[0].split('(').last().unwrap();
                {
                    let vl = vars.len() - 1;
                    vars[vl] = &vars[vl][0..vars[vl].len() - 1];
                }
                let mut x = func.len();
                while x > 0
                {
                    x -= 1;
                    if func[x].str_is(&fv.0)
                    {
                        let mut fv = fv.clone();
                        let mut i = 0;
                        while i < func_vars.len()
                        {
                            if vars.contains(&func_vars[i].0.as_str())
                            {
                                func_vars.remove(i);
                                continue;
                            }
                            i += 1;
                        }
                        let mut bracket = 0;
                        let mut k = 0;
                        let mut processed = Vec::new();
                        let mut last = 0;
                        for (i, n) in func[x + 2..].iter().enumerate()
                        {
                            if let Str(s) = n
                            {
                                if s == "(" || s == "{"
                                {
                                    bracket += 1
                                }
                                else if s == ")" || s == "}"
                                {
                                    if bracket == 0
                                    {
                                        if let Ok(n) = do_math(
                                            func[x + 2 + last..x + 2 + i].to_vec(),
                                            options,
                                            func_vars.clone(),
                                        )
                                        {
                                            processed.push(vec![n]);
                                        }
                                        else
                                        {
                                            let iden = format!(
                                                "@{}{}@",
                                                func_vars.len(),
                                                vars[processed.len()]
                                            );
                                            func_vars.push((
                                                iden.clone(),
                                                func[x + 2 + last..x + 2 + i].to_vec(),
                                            ));
                                            processed.push(vec![Str(iden)]);
                                        }
                                        k = i;
                                        break;
                                    }
                                    bracket -= 1;
                                }
                                else if s == "," && bracket == 0
                                {
                                    if let Ok(n) = do_math(
                                        func[x + 2 + last..x + 2 + i].to_vec(),
                                        options,
                                        func_vars.clone(),
                                    )
                                    {
                                        processed.push(vec![n]);
                                    }
                                    else
                                    {
                                        let iden = format!(
                                            "@{}{}@",
                                            func_vars.len(),
                                            vars[processed.len()]
                                        );
                                        func_vars.push((
                                            iden.clone(),
                                            func[x + 2 + last..x + 2 + i].to_vec(),
                                        ));
                                        processed.push(vec![Str(iden)]);
                                    }
                                    last = i + 1;
                                }
                            }
                        }
                        let mut i = 0;
                        while i < fv.1.len()
                        {
                            if let Str(s) = &fv.1[i]
                            {
                                for v in processed.iter().zip(vars.clone())
                                {
                                    if *s == v.1
                                    {
                                        fv.1.remove(i);
                                        fv.1.splice(i..i, v.0.clone());
                                        break;
                                    }
                                }
                            }
                            i += 1;
                        }
                        func.drain(x..=k + x + 2);
                        func.splice(x..x, fv.1.clone());
                    }
                }
            }
            else
            {
                let var = fv.0.split('(').last().unwrap();
                let var = &var[0..var.len() - 1];
                let mut x = func.len();
                while x > 0
                {
                    x -= 1;
                    if func[x].str_is(&fv.0)
                    {
                        let mut fv = fv.clone();
                        for (i, j) in func_vars.clone().iter().enumerate()
                        {
                            if j.0 == var
                            {
                                func_vars.remove(i);
                            }
                        }
                        let mut bracket = 0;
                        let mut k = 0;
                        for (i, n) in func[x + 2..].iter().enumerate()
                        {
                            if let Str(s) = n
                            {
                                if s == "(" || s == "{"
                                {
                                    bracket += 1
                                }
                                else if s == ")" || s == "}"
                                {
                                    if bracket == 0
                                    {
                                        k = i;
                                    }
                                    bracket -= 1;
                                }
                            }
                        }
                        let mut i = 0;
                        while i < func_vars.len()
                        {
                            if var == func_vars[i].0
                            {
                                func_vars.remove(i);
                                break;
                            }
                            i += 1;
                        }
                        let iden = format!("@{}{}@", func_vars.len(), var);
                        let mut i = 0;
                        while i < fv.1.len()
                        {
                            if let Str(s) = &fv.1[i]
                            {
                                if *s == var
                                {
                                    fv.1[i] = Str(iden.clone());
                                }
                            }
                            i += 1;
                        }
                        func_vars.push((iden.clone(), func[i + 2..=k + 1].to_vec()));
                        func.drain(x..=k + x + 2);
                        func.splice(x..x, fv.1.clone());
                    }
                }
            }
        }
    }
    do_math(func, options, func_vars)
}
pub fn gcd(mut x: Integer, mut y: Integer) -> Integer
{
    while x != y
    {
        if x > y
        {
            x -= y.clone()
        }
        else
        {
            y -= x.clone()
        }
    }
    x
}
//simpsons rule
// pub fn incomplete_beta(x: Complex, a: Complex, b: Complex) -> Complex
// {
//     let mut last = Complex::new(a.prec());
//     let i = 12u32;
//     let mut area = Complex::new(a.prec());
//     let n: Complex = x.clone() / 2.pow(i);
//     for k in 1..=2.pow(i)
//     {
//         let g: Complex = k * n.clone();
//         let f: Complex = 1 - g.clone();
//         let num: Complex = g.pow(a.clone()) * f.pow(b.clone());
//         let g: Complex = (k * 2 - 1) * n.clone() / 2;
//         let f: Complex = 1 - g.clone();
//         let mid: Complex = g.pow(a.clone()) * f.pow(b.clone());
//         area += (last + 4 * mid + num.clone()) * x.clone() / (3 * 2.pow(i + 1));
//         last = num;
//     }
//     area
// }
pub fn incomplete_beta(x: Complex, a: Complex, b: Complex) -> Complex
{
    if x.real() > &((a.real().clone() + 1) / (a.real() + b.real().clone() + 2))
    {
        (gamma(a.real()) * gamma(b.real()) / gamma(&(a.real() + b.real().clone())))
            - incomplete_beta(1 - x, b, a)
    }
    else
    {
        let f: Complex = 1 - x.clone();
        x.clone().pow(a.clone()) * f.pow(b.clone())
            / (a.clone() * (1 + incomplete_beta_recursion(x, a, b, 1, 10)))
    }
}
fn incomplete_beta_recursion(x: Complex, a: Complex, b: Complex, iter: usize, max: usize)
    -> Complex
{
    if iter == max
    {
        Complex::new(x.prec())
    }
    else if iter % 2 == 1
    {
        let m = (iter - 1) / 2;
        (-x.clone() * (a.clone() + m) * (a.clone() + b.clone() + m)
            / ((a.clone() + (2 * m)) * (a.clone() + (2 * m) + 1)))
            / (1 + incomplete_beta_recursion(x, a, b, iter + 1, max))
    }
    else
    {
        let m = iter / 2;
        (x.clone() * m * (b.clone() - m) / ((a.clone() + (2 * m)) * (a.clone() + (2 * m) - 1)))
            / (1 + incomplete_beta_recursion(x, a, b, iter + 1, max))
    }
}
pub fn incomplete_gamma(s: Complex, z: Complex) -> Complex
{
    // let prec = Float::with_val(z.prec().0, 0.1).pow(z.prec().0 / 2);
    // let mut last: Complex = incomplete_gamma_recursion(s.clone(), z.clone(), 0, 1);
    // let mut num = incomplete_gamma_recursion(s.clone(), z.clone(), 0, 2);
    // for m in 3..100
    // {
    //     if (num.clone() - last.clone()).abs().real() > &prec
    //     {
    //         last = num.clone();
    //         num = incomplete_gamma_recursion(s.clone(), z.clone(), 0, m);
    //     }
    //     else
    //     {
    //         break;
    //     }
    // }
    incomplete_gamma_recursion(s, z, 0, 100)
}
fn incomplete_gamma_recursion(s: Complex, z: Complex, iter: usize, max: usize) -> Complex
{
    if iter == max
    {
        Complex::with_val(s.prec(), 1)
    }
    else if iter == 0
    {
        (z.clone().pow(s.clone()) / z.clone().exp()) / incomplete_gamma_recursion(s, z, 1, max)
    }
    else if iter % 2 == 1
    {
        z.clone()
            + ((iter.div_ceil(2) - s.clone()) / incomplete_gamma_recursion(s, z, iter + 1, max))
    }
    else
    {
        1 + (iter.div_ceil(2) / incomplete_gamma_recursion(s, z, iter + 1, max))
    }
}
pub fn subfactorial(z: Complex) -> Complex
{
    //let prec = Float::with_val(z.prec().0, 0.1).pow(z.prec().0 / 2);
    // let mut last: Complex = subfactorial_recursion(z.clone(), 0, 1);
    // let mut num = subfactorial_recursion(z.clone(), 0, 2);
    // for m in 3..100
    // {
    //     if (num.clone() - last.clone()).abs().real() > &prec
    //     {
    //         last = num.clone();
    //         num = subfactorial_recursion(z.clone(), 0, m);
    //     }
    //     else
    //     {
    //         break;
    //     }
    // }
    subfactorial_recursion(z.clone(), 0, 100)
        + gamma(&(z.real().clone() + 1)) / Complex::with_val(z.prec(), 1).exp()
}
fn subfactorial_recursion(z: Complex, iter: usize, max: usize) -> Complex
{
    if iter == max
    {
        Complex::with_val(z.prec(), 1)
    }
    else if iter == 0
    {
        Complex::with_val(z.prec(), -1).pow(z.clone()) / subfactorial_recursion(z, 1, max)
    }
    else
    {
        (z.clone() + iter + 1) - iter / subfactorial_recursion(z, iter + 1, max)
    }
}
pub fn length(
    func: Vec<NumStr>,
    mut func_vars: Vec<(String, Vec<NumStr>)>,
    options: Options,
    var: String,
    start: Complex,
    end: Complex,
    points: usize,
) -> Result<Complex, &'static str>
{
    let delta: Complex = (end - start.clone()) / points;
    func_vars.push((var.clone(), vec![Num(start.clone())]));
    let mut last = do_math(func.clone(), options, func_vars.clone())?;
    func_vars.pop();
    let mut length = Complex::new(options.prec);
    for n in 1..=points
    {
        let x: Complex = start.clone() + (n * delta.clone());
        func_vars.push((var.clone(), vec![Num(x.clone())]));
        let y = do_math(func.clone(), options, func_vars.clone())?;
        func_vars.pop();
        let t: Complex = match (last, y.clone())
        {
            (Num(last), Num(y)) => delta.clone().pow(2) + (y.clone() - last).pow(2),
            (Vector(last), Vector(y)) if last.len() == 2 =>
            {
                (last[0].clone() - y[0].clone()).pow(2) + (last[1].clone() - y[1].clone()).pow(2)
            }
            (Vector(last), Vector(y)) if last.len() == 3 =>
            {
                (last[0].clone() - y[0].clone()).pow(2)
                    + (last[1].clone() - y[1].clone()).pow(2)
                    + (last[2].clone() - y[2].clone()).pow(2)
            }
            (_, _) => return Err("not supported arc length data"),
        };
        last = y;
        length += t.sqrt()
    }
    Ok(length)
}
pub fn area(
    func: Vec<NumStr>,
    mut func_vars: Vec<(String, Vec<NumStr>)>,
    options: Options,
    var: String,
    start: Complex,
    end: Complex,
    points: usize,
) -> Result<Complex, &'static str>
{
    func_vars.push((var.clone(), vec![Num(start.clone())]));
    let mut last = do_math(func.clone(), options, func_vars.clone())?;
    func_vars.pop();
    let delta: Complex = (end - start.clone()) / points;
    let mut area = Complex::new(options.prec);
    for n in 1..=points
    {
        let h: Complex = delta.clone() / 3;
        let x: Complex = start.clone() + ((n - 1) * delta.clone());
        func_vars.push((var.clone(), vec![Num(start.clone() + (n * delta.clone()))]));
        let y = do_math(func.clone(), options, func_vars.clone())?;
        func_vars.pop();
        func_vars.push((var.clone(), vec![Num(x.clone() + h.clone())]));
        let ym1 = do_math(func.clone(), options, func_vars.clone())?;
        func_vars.pop();
        func_vars.push((var.clone(), vec![Num(x + 2 * h.clone())]));
        let ym2 = do_math(func.clone(), options, func_vars.clone())?;
        func_vars.pop();
        match (last, y.clone(), ym1, ym2)
        {
            (Num(last), Num(y), Num(ym1), Num(ym2)) =>
            {
                area += 3 * h * (last + y + 3 * (ym1 + ym2)) / 8
            }
            // (Vector(last), Vector(y), Vector(ym1), Vector(ym2))
            //     if last.len() == 2 =>
            // {
            //     area += 3
            //         * h
            //         * (last[1].clone()
            //             + y[1].clone()
            //             + 3 * (ym1[1].clone() + ym2[1].clone()))
            //         / 8
            // }
            (_, _, _, _) => return Err("not supported area data"),
        };
        last = y;
    }
    Ok(area)
}
pub fn slope(
    func: Vec<NumStr>,
    mut func_vars: Vec<(String, Vec<NumStr>)>,
    options: Options,
    var: String,
    point: Complex,
    combine: bool,
) -> Result<NumStr, &'static str>
{
    let h = Complex::with_val(options.prec, 0.5).pow(options.prec.0 / 2);
    func_vars.push((var.clone(), vec![Num(point.clone())]));
    let n1 = do_math(func.clone(), options, func_vars.clone())?;
    func_vars.pop();
    func_vars.push((var.clone(), vec![Num(point + h.clone())]));
    let n2 = do_math(func, options, func_vars.clone())?;
    func_vars.pop();
    match (n1, n2)
    {
        (Num(n1), Num(n2)) => Ok(Num((n2 - n1) / h)),
        (Vector(n1), Vector(n2)) if !combine => Ok(Vector(
            n2.iter()
                .zip(n1)
                .map(|(f, i)| (f - i) / h.clone())
                .collect::<Vec<Complex>>(),
        )),
        (Vector(n1), Vector(n2)) if n1.len() == 2 => Ok(Num(
            (n2[1].clone() - n1[1].clone()) / (n2[0].clone() - n1[0].clone())
        )),
        (Vector(n1), Vector(n2)) if n1.len() == 3 => Ok(Vector(vec![
            (n2[2].clone() - n1[2].clone()) / (n2[0].clone() - n1[0].clone()),
            (n2[2].clone() - n1[2].clone()) / (n2[1].clone() - n1[1].clone()),
        ])),
        (_, _) => Err("not supported slope data"),
    }
}
//https://github.com/IstvanMezo/LambertW-function/blob/master/complex%20Lambert.cpp
pub fn lambertw(z: Complex, k: isize) -> Complex
{
    if z.is_zero()
    {
        return if k == 0
        {
            Complex::new(z.prec())
        }
        else
        {
            -Complex::with_val(z.prec(), Infinity)
        };
    }
    let prec = Float::with_val(z.prec().0, 0.1).pow(z.prec().0 / 2);
    let mut w = initpoint(z.clone(), k);
    let mut wprev = w.clone();
    {
        let zexp = w.clone().exp();
        let zexpz = w.clone() * zexp.clone();
        let zexpz_d = zexp.clone() + zexpz.clone();
        let zexpz_dd = (2 * zexp) + zexpz.clone();
        w -= 2 * ((zexpz.clone() - z.clone()) * zexpz_d.clone())
            / ((2 * zexpz_d.pow(2)) - ((zexpz - z.clone()) * zexpz_dd))
    }
    for _ in 0..50
    {
        if (w.clone() - wprev.clone()).abs().real() > &prec
        {
            wprev = w.clone();
            let zexp = w.clone().exp();
            let zexpz = w.clone() * zexp.clone();
            let zexpz_d = zexp.clone() + zexpz.clone();
            let zexpz_dd = (2 * zexp) + zexpz.clone();
            w -= 2 * ((zexpz.clone() - z.clone()) * zexpz_d.clone())
                / ((2 * zexpz_d.pow(2)) - ((zexpz - z.clone()) * zexpz_dd))
        }
        else
        {
            break;
        }
    }
    w
}
fn initpoint(z: Complex, k: isize) -> Complex
{
    let pi = Float::with_val(z.prec().0, Pi);
    let e = Float::with_val(z.prec().0, 1).exp();
    {
        let test: Complex = z.clone() + (1 / e.clone());
        if test.abs().real() <= &1
        {
            let p1: Complex = 2 * e * z.clone() + 2;
            let p = p1.clone().sqrt();
            if k == 0
            {
                return p.clone() - (p1 / 3) + ((11 * p.pow(3)) / 72) - 1;
            }
            else if (k == 1 && z.imag() < &0) || (k == -1 && z.imag() > &0)
            {
                return -1 - p.clone() - (p1 / 3) - ((11 * p.pow(3)) / 72);
            }
        }
    }
    {
        let test: Complex = z.clone() - 0.5;
        if test.abs().real() <= &0.5
        {
            if k == 0
            {
                return (0.35173371 * (0.1237166 + 7.061302897 * z.clone()))
                    / (2 + 0.827184 * (1 + 2 * z));
            }
            else if k == -1
            {
                return (Complex::with_val(z.prec(), (-2.2591588985, -4.22096))
                    * (Complex::with_val(z.prec(), (-14.073271, 33.767687754)) * z.clone()
                        + Complex::with_val(z.prec(), (-12.7127, 19.071643))
                            * (1 + 2 * z.clone())))
                    / (2 + Complex::with_val(z.prec(), (-17.23103, 10.629721)) * (1 + 2 * z));
            }
        }
    }
    let two_pi_k_i = Complex::with_val(z.prec(), (0, 2 * pi * k));
    let zln = z.clone().ln() + two_pi_k_i;
    zln.clone() - zln.ln()
}