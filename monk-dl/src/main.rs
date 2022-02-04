fn main() {
    let url = "https://arxiv.org/abs/2202.00667";

    let mime = mime_guess::from_path(url);

    println!("{:?}", mime.first());
}
