use std::cmp;

#[derive(PartialEq, Debug)]
pub struct Fraction {
    numerator: i64,
    denominator: u64,
}

fn gcd(mut x: u64, mut y: u64) -> u64 {
    println!("wtf?");
    while x != 0 && y !=0 {
        println!("{}, {}",x, y);
        if x > y {
            x -= y;
        } else {
            y -= x;
        }
    }
    println!("{}, {}",x, y);
    if x == 0 {
        y
    } else {
        x
    }
}

impl Fraction {
    pub fn new(numerator: i64, denominator: u64) -> Option<Fraction> {
        if 0 == denominator {
            return None;
        }
        let mut new_fraction = Fraction {
            numerator,
            denominator,
        };
        new_fraction.simplify();
        Some(new_fraction)
    }
    fn simplify(&mut self) {
        if self.numerator == 0 {
            self.denominator = 1;
            return;
        }
        let gcd = gcd(self.numerator.abs() as u64, self.denominator);
        self.numerator /= gcd as i64;
        self.denominator /= gcd;
    }
    pub fn contents(&self) -> (i64, i64) {
        (self.numerator, self.denominator as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fractions_simplify() {
        let half = Fraction::new(1,2);
        let alt_half = Fraction::new(2,4);
        assert_eq!(half,alt_half);
    }
    #[test]
    fn negatives_simplify() {
        let neg_third = Fraction::new(-1,3);
        let alt_neg_third = Fraction::new(-4,12);
        assert_eq!(neg_third,alt_neg_third);
    }
    #[test]
    fn zeros_equal() {
        let zero = Fraction::new(0,2);
        let other_zero = Fraction::new(0,3);
        assert_eq!(zero, other_zero);
    }
    #[test]
    fn reject_nan() {
        let nan = Fraction::new(2,0);
        assert_eq!(nan, None);
    }
}
