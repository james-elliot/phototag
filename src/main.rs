/*
exiftool -geotag=Louisiane.gpx -geosync=+6:00:00 .
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

fn get_latlon(path: &str) -> (f64,f64,String) {
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).unwrap();
    let mut lat  =0.;
    let mut lon =0.;
    let mut s= "".to_string();
    for f in exif.fields() {
	match f.tag.description() {
	    Some(t) => {
//		println!("{:?} {}",t,f.display_value().with_unit(&exif).to_string());
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
		if t.eq("Date and time of original data generation")
		{
		    s = f.display_value().with_unit(&exif).to_string();
		}
	    },
	    None => {}
	}
    }
    return (lat,lon,s);
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

    //    darktable-cli --height 1080 L1004100.DNG toto.jpg
    let path = "L1004163.DNG";
    let status = std::process::Command::new("/usr/bin/darktable-cli")
        .args([
	    "--height", "1080",
	    &path,
	    "/mnt/ramfs/toto.jpg"
	])
        .status()
        .expect("failed to execute process");
    println!("process finished with: {status}");
    
    let (lat,lon,date)=get_latlon(&"/mnt/ramfs/toto.jpg");
    let r = tab.iter().min_by_key(
	|x| {
	    let d= dist(lat,lon,x.lat,x.lon);
//	    println!("{} {} {} {} {}",lat,lon,x.lat,x.lon,d);
	    d as i64
	}
    ).unwrap();
//    println!("{} {} {}",date,r.city,r.country);
    let v: Vec<&str> = date.split(' ').collect();
    let s = "label:".to_owned()+v[0]+"\n"+v[1]+"\n"+&r.city+"\n"+&r.country;
    println!("{}",s);
//    convert -background black -fill white -size 480x1080  -gravity center  label:"Ville Plate\nUnited States"   label_size.jpg
    let status = std::process::Command::new("/usr/bin/convert")
        .args([
	    "-background", "black",
	    "-fill", "white",
	    "-size", "480x1080",
	    "-gravity", "center",
	    &s  ,
	    "/mnt/ramfs/label_size.jpg"
	])
        .status()
        .expect("failed to execute process");
    println!("process finished with: {status}");

//    convert +append label_size.jpg toto.jpg out.jpg
    let status = std::process::Command::new("/usr/bin/convert")
        .args([
	    "+append",
	    "/mnt/ramfs/toto.jpg",
	    "/mnt/ramfs/label_size.jpg",
	    "/mnt/f/jpegs/out.jpg"
	])
        .status()
        .expect("failed to execute process");
    println!("process finished with: {status}");

    let status = std::process::Command::new("/usr/bin/rm")
        .args([
	    "/mnt/ramfs/toto.jpg",
	    "/mnt/ramfs/label_size.jpg"
	])
        .status()
        .expect("failed to execute process");
    println!("process finished with: {status}");
}
