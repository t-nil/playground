use iter_num_tools::arange;
extern crate nalgebra as na;

/*pub fn plot<Num: Float = f32, Iter: Iterator<Item = Num>, F: Fn(Num, Num) -> Num>(
    f: F,
    min: Num,
    max: Num,
    step: Num,
) -> impl Iterator<Item = Num> {
    let x = (min..max);
    Into::<Iter>::into(arange(x, step)).map(|x| (x, f(x)))
}*/

pub fn plot<'a, F: 'a + Fn(f64) -> f64>(
    f: F,
    min: f64,
    max: f64,
    step: f64,
) -> impl 'a + Iterator<Item = (f64, f64)> {
    let range = min..max;
    arange(range, step).map(move |x| (x, f(x)))
}
// TODO div0 and such: return option
