/*
sudo mount -t tmpfs -o size=1g tmpfs /mnt/ramfs
convert -background black -fill white -size 480x1080  -gravity center  label:"Ville Plate\nUnited States"   label_size.jpg
darktable-cli --height 1080 L1004100.DNG toto.jpg
convert +append label_size.jpg toto.jpg out.jpg
*/

use exif;
use csv;
use serde;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
#[allow(non_snake_case)]
struct CityCsv {
    ASCII_Name : String,
    Country_name : String,
    Coordinates : String
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct City {
    city : String,
    country : String,
    lat : f64,
    lon : f64
}


fn read_cities(file_path: &str) -> Vec<City> {
    let mut tab = Vec::new();
    let file = std::fs::File::open(file_path).unwrap();
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    for result in rdr.deserialize::<CityCsv>() {
	let r = result.unwrap();
	let v: Vec<&str> = r.Coordinates.split(',').collect();
	let lat = v[0].parse::<f64>().unwrap();
	let lon = v[1].parse::<f64>().unwrap();
	let e = City {
	    city : r.ASCII_Name,
	    country : r.Country_name,
	    lat : lat, lon : lon};
	tab.push(e);
    }
    tab
}

fn get_latlon(path: &str) -> (f64,f64) {
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).unwrap();
    let mut lat  =0.;
    let mut lon =0.;
    for f in exif.fields() {
	match f.tag.description() {
	    Some(t) => {
		if t.eq("Latitude") {
		    let s = f.display_value().with_unit(&exif).to_string();
		    let v: Vec<&str> = s.split(' ').collect();
		    lat = v[0].parse::<f64>().unwrap()+v[2].parse::<f64>().unwrap()/60.+
			v[4].parse::<f64>().unwrap()/3600.;
		    if v[6].eq("S") {
			lat = -lat;
		    }
		}
		if t.eq("Longitude") {
		    let s = f.display_value().with_unit(&exif).to_string();
		    let v: Vec<&str> = s.split(' ').collect();
		    lon = v[0].parse::<f64>().unwrap()+v[2].parse::<f64>().unwrap()/60.+
			v[4].parse::<f64>().unwrap()/3600.;
		    if v[6].eq("W") {
			lon = -lon;
		    }
		}
	    },
	    None => {}
	}
    }
    return (lat,lon);
}

fn deg2rad(deg:f64) -> f64 {
  return deg * std::f64::consts::PI/180.;
}

fn dist(lat1:f64,lon1:f64,lat2:f64,lon2:f64) -> f64{
  let r = 6371.; // Radius of the earth in km
  let dlat = deg2rad(lat2-lat1);  
  let dlon = deg2rad(lon2-lon1); 
  let a = 
    (dlat/2.).sin() * (dlat/2.).sin() +
    (deg2rad(lat1)).cos() * (deg2rad(lat2)).cos() * 
    (dlon/2.).sin() * (dlon/2.).sin(); 
  let c = 2. * a.sqrt().atan2((1.-a).sqrt()); 
  let d = r * c; // Distance in km
  return d;
}

fn main() {
    let tab = read_cities("cities.csv");
    let (lat,lon)=get_latlon(&"L1004163.DNG");
    let r = tab.iter().min_by_key(
	|x| {
	    let d= dist(lat,lon,x.lat,x.lon);
//	    println!("{} {} {} {} {}",lat,lon,x.lat,x.lon,d);
	    d as i64
	}
    ).unwrap();
    println!("{} {} {:?} {}",lat,lon,r,dist(lat,lon,r.lat,r.lon));
}
