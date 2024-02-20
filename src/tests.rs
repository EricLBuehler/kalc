use crate::{
    complex::NumStr::{Num, Str},
    load_vars::get_vars,
    math::do_math,
    parse::input_var,
    Options,
};
use rug::{float::Constant::Pi, Complex};
#[test]
fn test_math()
{
    let output = input_var(
        "pi+tau*e/2i^(sqrt(2))/3*3-log(2-2i,-3+i)+sqrt(2)^(sqrt(2))",
        get_vars(Options::default()),
        &mut Vec::new(),
        &mut 0,
        Options::default(),
        false,
        false,
        0,
        Vec::new(),
    )
    .unwrap();
    let expected = vec![
        Num(Complex::with_val(512, Pi)),
        Str("+".to_string()),
        Num(2 * Complex::with_val(512, Pi)),
        Str("*".to_string()),
        Num(Complex::with_val(512, 1).exp()),
        Str("/".to_string()),
        Num(Complex::with_val(512, 2)),
        Str("*".to_string()),
        Num(Complex::with_val(512, (0.0, 1))),
        Str("^".to_string()),
        Str("(".to_string()),
        Str("sqrt".to_string()),
        Str("(".to_string()),
        Num(Complex::with_val(512, 2)),
        Str(")".to_string()),
        Str(")".to_string()),
        Str("/".to_string()),
        Num(Complex::with_val(512, 3)),
        Str("*".to_string()),
        Num(Complex::with_val(512, 3)),
        Str("-".to_string()),
        Str("log".to_string()),
        Str("(".to_string()),
        Num(Complex::with_val(512, 2)),
        Str("-".to_string()),
        Num(Complex::with_val(512, 2)),
        Str("*".to_string()),
        Num(Complex::with_val(512, (0, 1))),
        Str(",".to_string()),
        Num(Complex::with_val(512, -3)),
        Str("+".to_string()),
        Num(Complex::with_val(512, (0, 1))),
        Str(")".to_string()),
        Str("+".to_string()),
        Str("sqrt".to_string()),
        Str("(".to_string()),
        Num(Complex::with_val(512, 2)),
        Str(")".to_string()),
        Str("^".to_string()),
        Str("(".to_string()),
        Str("sqrt".to_string()),
        Str("(".to_string()),
        Num(Complex::with_val(512, 2)),
        Str(")".to_string()),
        Str(")".to_string()),
    ];
    let out = do_math(output.0, Options::default(), Vec::new())
        .unwrap()
        .num()
        .unwrap();
    let answer = do_math(expected, Options::default(), Vec::new())
        .unwrap()
        .num()
        .unwrap();
    assert_eq!(out.real().to_string(), answer.real().to_string());
    assert_eq!(out.imag().to_string(), answer.imag().to_string());
    assert_eq!(&out.real().to_string()[..20], "2.009877988310399125");
    assert_eq!(&out.imag().to_string()[..20], "4.535664430265577075");
}