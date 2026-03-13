fn main() {
    let img = image::open("test.jpg");
    println!("{:?}", color_thief::get_palette);
}
