#[allow(dead_code)]
pub fn gen_print(w: u32, h: u32) {
	println!("P3\n{} {}\n255", w, h);
	for j in (0..h-1).rev() {
		for i in 0..w {
			let r = (256.0 * (i as f32 / w as f32)) as u32;
			let g = (256.0 * (j as f32 / h as f32)) as u32;
			let b = (256.0 * 0.5) as u32;
			println!("{} {} {}", r, g, b);
		}
	}
}
