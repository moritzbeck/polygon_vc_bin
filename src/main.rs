extern crate polygon;
extern crate vcdim;

use polygon::generate::Mode;
use vcdim::*;
use std::io::Write;
use std::fs::File;
//use std::fs::OpenOptions;

fn is_interesting(vcd: &VcDim) -> bool {
    let vc_dim = vcd.vc_dimension();
    if vc_dim >= 5 {
        true
    } else if vc_dim == 4 {
        false
    } else {
        false
    }
    // vc_dim > 4 || has_unexpected_visibility_string(vcd)
}

fn main() {
    let default_gen_mode = Mode::QuickStarLike;;
    // simple command line argument handling
    let mut args = std::env::args();
    let n = if args.len() >= 2 {
        args.nth(1).expect("Please give an argument!").parse().expect("The first argument should be a natural number.")
    } else { 20 };
    let count = if let Some(cnt) = args.next() {
        cnt.parse().expect("The second argument (if provided) should be a natural number.")
    } else { 10 };
    let gen_mode = if let Some(m) = args.next() {
        if m.starts_with("--mode=") {
            match &m[7..] {
                "2opt" => Mode::TwoOptLike,
                "quickstar" => Mode::QuickStarLike,
                "xmonotone" => Mode::Xmonotone,
                "monotone" => Mode::Xmonotone,
                "xmonotonegap" => Mode::XmonotoneGap,
                "monotonegap" => Mode::XmonotoneGap,
                _ => {
                    println!("Generation mode not recognised: {}\nPossible Values: 2opt, quickstar, xmonotone, xmonotonegap", &m[7..]);
                    return;
                }
            }
        } else {
            println!("Unrecognised argument: {}", m);
            return;
        }
    } else { default_gen_mode };

    println!("Visibility VC-dimension of Polygons\n===================================\n\nCreating {} Polygons with {} vertices", count, n);
    println!("Mode: {:?}", gen_mode);

    let mut stdout = std::io::stdout();

    let mut vcd = VcDim::with_random_polygon(n, gen_mode);

    for i in 0..count {
        let vc_dim = vcd.vc_dimension();
        // minimize first?!
        if is_interesting(&vcd) {
            vcd.export_ipe(File::create(format!("out/vc{}-n{}_{:03}.ipe", vc_dim, n, i)).unwrap(), 1f64).unwrap();
        }
        print!("{}", vc_dim);
        stdout.flush().expect("Couldn't flush stdout");

        if i != count-1 {
            vcd.randomize_polygon(gen_mode);
        }
    }
    println!("");
}
