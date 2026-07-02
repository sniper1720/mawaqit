use mawaqit::prelude::*;

fn main() {
    let locations = [
        ("New York", Coordinates::new(40.7128, -74.0060)),
        ("Mecca", Coordinates::new(21.4225, 39.8262)),
        ("Jakarta", Coordinates::new(-6.2088, 106.8456)),
        ("London", Coordinates::new(51.5074, -0.1278)),
        ("Tokyo", Coordinates::new(35.6762, 139.6503)),
    ];

    println!("Qibla direction from major cities:");
    for (name, coords) in &locations {
        let qiblah = Qiblah::new(*coords);
        println!("  {name:<12}  {qibla:>7.2}°", qibla = qiblah.value());
    }
}
