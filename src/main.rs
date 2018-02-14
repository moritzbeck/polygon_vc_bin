extern crate polygon;
extern crate vcdim;
extern crate rand;

use polygon::generate::Mode;
use polygon::{Point, Polygon};
use vcdim::*;
use std::io::Write;
use std::fs::File;
use rand::distributions::{IndependentSample, Range};
//use std::fs::OpenOptions;

fn minimize_w_subset(polygon: &Polygon, sh_set: &[Point]) -> VcDim {
    let mut vcd = VcDim::new(polygon.clone());
    let mut points = vcd.points().to_vec();
    let mut polygon;

    assert!(vcd.is_shattered(sh_set));
    assert!(vcd.polygon().is_simple());

    let mut rng = rand::thread_rng();
    let mut minimized = false;
    while !minimized {
        if points.len() == 3 { break; }
        minimized = true;
        let range = Range::new(0, points.len());
        let rand_offset = range.ind_sample(&mut rng);
        for i in 0..points.len() { // go around the polygon
            let idx = (i + rand_offset) % points.len();
            let pt = points.remove(idx);
            polygon = Polygon::from_points(&points);
            if !polygon.is_simple() {
                points.insert(idx, pt);
                continue;
            }
            vcd = VcDim::new(polygon); // this is quite un-performant (I guess) as this recomputes the entire visibility matrix.
            if vcd.is_shattered(&sh_set) {
                minimized = false;
                break;
            } else {
                points.insert(idx, pt);
            }
        }
    }

    vcd = VcDim::new(Polygon::from_points(&points));
    vcd
}
fn is_interesting(vcd: &VcDim) -> bool {
    let vc_dim = vcd.vc_dimension();
    if vc_dim >= 5 {
        true
    } else if vc_dim == 4 {
        has_unexpected_visibility_string(vcd)
    } else {
        false
    }
}
fn has_unexpected_visibility_string(vcd: &VcDim) -> bool {
	let vstr = visibility_structure_simplified(vcd);

	vcd.vc_dimension() == 4 && vcd.is_small()
		&& vstr != "aaaa" && vstr != "abbc"
}
fn visibility_structure_simplified(vcd: &VcDim) -> String {
    let pts = vcd.points();
    let sh = vcd.max_shattered_subset();
    let mut chars = vcd.visible().iter().enumerate().filter_map(|(i,v)| {
        let c = v.iter().enumerate()
            .filter(|&(j, sees_i)| { sh.contains(&pts[j]) && *sees_i })
            .count();
        if sh.contains(&pts[i]) {
            Some((96 + c as u8) as char) // 1 -> 'a', 2 -> 'b', etc.
        } else {
            None
        }
    }).collect::<Vec<char>>();
    chars.sort();
    chars.into_iter().collect::<String>()
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
    let mut export_all = false;
    let mut gen_mode = default_gen_mode;
    let mut minimize = false;
    for arg in args {
        if arg.starts_with("--mode=") {
            gen_mode = match &arg[7..] {
                "2opt" => Mode::TwoOptLike,
                "quickstar" => Mode::QuickStarLike,
                "xmonotone" => Mode::Xmonotone,
                "monotone" => Mode::Xmonotone,
                "xmonotonegap" => Mode::XmonotoneGap,
                "monotonegap" => Mode::XmonotoneGap,
                _ => {
                    println!("Generation mode not recognised: {}\nPossible Values: 2opt, quickstar, xmonotone, xmonotonegap", &arg[7..]);
                    return;
                }
            };
        } else if arg == "--export-all" {
            export_all = true;
        } else if arg == "--minimize" {
            minimize = true;
        } else {
            println!("Unrecognised argument: {}", arg);
            return;
        }
    }

    println!("Visibility VC-dimension of Polygons\n===================================\n\nCreating {} Polygons with {} vertices", count, n);
    println!("Mode: {:?}", gen_mode);

    let mut stdout = std::io::stdout();

    let mut vcd = VcDim::with_random_polygon(n, gen_mode);

	let mut minimum_polygons = 0;
    for i in 0..count {
        let vc_dim = vcd.vc_dimension();
        // minimize first?!
        if minimize {
            let subset = vcd.max_shattered_subset();
            vcd = minimize_w_subset(vcd.polygon(), &subset);
            if vcd.vc_dimension() == 4 && vcd.is_small() {
                minimum_polygons += 1;
            }
        }
        if export_all || is_interesting(&vcd) {
            vcd.export_ipe(File::create(format!("out/vc{}-n{}_{:03}.ipe", vc_dim, n, i)).unwrap(), 1f64).unwrap();
        }
        print!("{}", vc_dim);
        stdout.flush().expect("Couldn't flush stdout");

        if i != count-1 {
            if minimize {
                vcd = VcDim::with_random_polygon(n, gen_mode);
            } else {
	            vcd.randomize_polygon(gen_mode);
	        }
        }
    }
    println!("\nFound {} minimum polygons with d = 4.", minimum_polygons);
}
