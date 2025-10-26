/*
Dont forget if you need a ramdisk
sudo mount -t tmpfs -o size=1g tmpfs /mnt/ramfs

To modify time and geotag:
exiftool -geotag=Louisiane.gpx -geosync=+6:00:00 .
*/

#[derive(Debug, serde::Deserialize)]
#[allow(non_snake_case)]
struct CityCsv {
    ASCII_Name: String,
    Country_name: String,
    Coordinates: String,
}

#[derive(Debug)]
struct City {
    city: String,
    country: String,
    lat: f64,
    lon: f64,
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
        tab.push(City {
            city: r.ASCII_Name,
            country: r.Country_name,
            lat,
            lon,
        });
    }
    tab
}

#[derive(Debug)]
struct DateTime {
    year:String,
    month:String,
    day:String,
    hour:String,
    minute:String,
    second:String
}

fn test_latlon(path: &str) -> Option<(f64,f64,DateTime,i32,i32)> {
    let meta = rexiv2::Metadata::new_from_path(path);
    match meta {
	Ok(res) => {
	    let width = res.get_pixel_width();
	    let height = res.get_pixel_height();
//	    eprintln!("width:{:?}", width);
//	    eprintln!("height:{:?}", height);
	    let d =
		match res.get_tag_string("Exif.Photo.DateTimeOriginal") {
		    Ok(date)=>{
//			eprintln!("{:?}",date);
			let v: Vec<&str> = date.split(' ').collect();
			let v0: Vec<&str> = v[0].split(':').collect();
//			let v0:Vec<i64> =    u0.iter().map(|s| s.parse::<i64>().unwrap()).collect();
			let v1: Vec<&str> = v[1].split(':').collect();
//			let v1:Vec<i64> =    u1.iter().map(|s| s.parse::<i64>().unwrap()).collect();
			Some(DateTime{year:v0[0].to_string(),month:v0[1].to_string(),day:v0[2].to_string(),hour:v1[0].to_string(),minute:v1[1].to_string(),second:v1[2].to_string()})
		    },
		    Err(_)=> None
		};
	    if let Some(date) = d {
//		eprintln!("{:?}",date);
		if let Some(gps)=res.get_gps_info() {
//		    eprintln!("{:?}",gps);
		    return Some((gps.latitude,gps.longitude,date,width,height));
		}
		else {
		    eprintln!("No date for {:?}",path);
		    return None;
		}
	    }
	    else {
		eprintln!("No gps for {:?}",path);
		return None;
	    }
	},
	Err(_)=> {
	    eprintln!("No metadata for {:?}",path);
	    return None
	}
    }
}
    
fn get_latlon(path: &str) -> Option<(f64, f64, String,i64,i64)> {
    test_latlon(path);
    let file = std::fs::File::open(path).unwrap();
    let mut bufreader = std::io::BufReader::new(&file);
    let exif_res = exif::Reader::new().read_from_container(&mut bufreader);
    match exif_res {
	Err(_) => None,
	Ok(exif) => {
	    let mut lat = 0.;
	    let mut lon = 0.;
	    let mut width = 0;
	    let mut height = 0;
	    let mut s1 = "".to_string();
//	    let mut s2 = "".to_string();
	    for f in exif.fields() {
		if let Some(t) = f.tag.description() {
		    eprintln!("{:?} {}",t,f.display_value().with_unit(&exif).to_string());
		    if t.eq("Latitude") {
			let s = f.display_value().with_unit(&exif).to_string();
			let v: Vec<&str> = s.split(' ').collect();
			lat = v[0].parse::<f64>().unwrap()
			    + v[2].parse::<f64>().unwrap() / 60.
			    + v[4].parse::<f64>().unwrap() / 3600.;
			if v[6].eq("S") {
			    lat = -lat;
			}
		    }
		    if t.eq("Longitude") {
			let s = f.display_value().with_unit(&exif).to_string();
			let v: Vec<&str> = s.split(' ').collect();
			lon = v[0].parse::<f64>().unwrap()
			    + v[2].parse::<f64>().unwrap() / 60.
			    + v[4].parse::<f64>().unwrap() / 3600.;
			if v[6].eq("W") {
			    lon = -lon;
			}
		    }
		    if t.eq("Date and time of original data generation") {
			s1 = f.display_value().with_unit(&exif).to_string();
		    }
		    if  t.eq("Exif Image Width") {
			width = f.display_value().with_unit(&exif).to_string().parse().unwrap();
		    }
		    if  t.eq("PixelXDimension") {
			width = f.display_value().with_unit(&exif).to_string().parse().unwrap();
		    }
		    if  t.eq("Exif Image Height") {
			height = f.display_value().with_unit(&exif).to_string().parse().unwrap();
		    }
		    if  t.eq("PixelYDimension") {
			height = f.display_value().with_unit(&exif).to_string().parse().unwrap();
		    }
		}
	    }
//	    eprintln!("s1:{:?} lat:{:?} lon:{:?}",s1,s2,lat,lon);
	    if lat == 0.  {None}
	    else {Some((lat, lon, s1,width,height))}
	}
    }
}

fn deg2rad(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.
}

fn dist(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.; // Radius of the earth in km
    let dlat = deg2rad(lat2 - lat1);
    let dlon = deg2rad(lon2 - lon1);
    let a = (dlat / 2.).sin() * (dlat / 2.).sin()
        + (deg2rad(lat1)).cos() * (deg2rad(lat2)).cos() * (dlon / 2.).sin() * (dlon / 2.).sin();
    let c = 2. * a.sqrt().atan2((1. - a).sqrt());
    r * c // Distance in km
}

fn one(p: &std::path::Path, tab: &[City], vexts: &[String], output_dir: &str, tmp_dir: &str) {
    let label_name = tmp_dir.to_owned()+"/label.jpg";
    let jpeg_name = tmp_dir.to_owned()+"/photo.jpg";
    let _p1 = p.file_stem().and_then(std::ffi::OsStr::to_str);
    let p2 = p.extension().and_then(std::ffi::OsStr::to_str);

    if let Some(s)=p2 {
        let s = s.to_ascii_lowercase();
	let res = vexts.iter().find (|&x| s.eq(x));
	match res {
	    None => {
		eprintln!("Not the right extension{:?}",p);
                return
	    },
	    Some(_) => {}
	}
	
	let path = p.to_str().unwrap();
	if let Some((lat, lon, date,width,height)) = test_latlon(path) {
	    eprintln!("date:{:?} lat:{:?} lon:{:?} width:{:?} height:{:?}",date,lat,lon,width,height);
	    
	    if  s!="jpg" {
		eprintln!("Using darktable as the image is not a jpeg");
		let status = std::process::Command::new("/usr/bin/darktable-cli")
		    .args(["--width","1620","--height", "1080", path, &jpeg_name])
		    .status()
		    .expect("failed to execute process darktable-cli");
		if !status.success() {
		    eprintln!("process darktable finished with status {} for file {:?}",status,p);
		    return;
		}
	    }
	    else {
		if height != 1080 {
		    eprintln!("Using convert as the image height is not 1080");
		    let status = std::process::Command::new("/usr/bin/convert")
			.args([
			    "-resize",
			    "x1080",
			    &path,
			    &jpeg_name,
			])
			.status()
			.expect("failed to execute process convert");
		    if !status.success() {
			eprintln!("process convert finished with status {} for file {:?}",status,p);
			return;
		    }
		}
		else {
		    eprintln!("Using copy as the image height is 1080 and it is a jpg");
		    let status2 = std::fs::copy(path,&jpeg_name);
		    match status2 {
			Ok(_) =>(),
			Err(_)=> {
			    eprintln!("copy failed for file {:?}",p);
			    return;
			}
		    }
		}
	    }
	
            let r = tab
		.iter()
		.min_by_key(|x| dist(lat, lon, x.lat, x.lon) as i64)
		.unwrap();
//            let v: Vec<&str> = date.split(' ').collect();
//	    let v0: Vec<&str> = v[0].split('-').collect();
//	    let v1: Vec<&str> = v[1].split(':').collect();
	    let v0 = vec![&date.year,&date.month,&date.day];
	    let v1 = vec![&date.hour,&date.minute,&date.second];
            let s = "label:".to_owned() + &date.day + "/" + &date.month + "/" + &date.year + "\n" +
		&date.hour + ":" + &date.minute + ":" + &date.second + "\n" +
		&r.city + "\n" +
		&r.country;
            let status = std::process::Command::new("/usr/bin/convert")
		.args([
                    "-background",
                    "black",
                    "-fill",
                    "white",
                    "-size",
                    "300x1080",
                    "-gravity",
                    "center",
                    &s,
                    &label_name,
		])
		.status()
		.expect("failed to execute process convert");
            if !status.success() {
		eprintln!("process convert finished with status {} for file {:?}",status,p);
		return;
	    }
	    let s = output_dir.to_owned() + "/IMG_" + v0[0]+v0[1]+v0[2]+"_"+v1[0]+v1[1]+v1[2] + ".jpg";
            let status = std::process::Command::new("/usr/bin/convert")
		.args([
                    "+append",
                    &jpeg_name,
                    &label_name,
                    &s,
		])
		.status()
		.expect("failed to execute process append");
            if !status.success() {
		eprintln!("process append finished with status {} for file {:?}",status,p);
		return;
	    }
	    std::fs::remove_file(&jpeg_name).expect("Can't remove file");
	    std::fs::remove_file(&label_name).expect("Can't remove file");
	}
	else {
	    eprintln!("Can't get lat/lon for {:?}",p);
	}
    }
    else {
	eprintln!("No extension for {:?}",p);
    }
}

use argparse::{ArgumentParser, Store};
fn main() {
    let mut output_dir = "/mnt/f/jpegs".to_string();
    let mut cities = "cities.csv".to_string();
    let mut search_dir = ".".to_string();
    let mut nb_levels = 1;
    let mut exts = "jpg".to_string();
    let mut tmp_dir = "/tmp".to_string();
    
    { // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Build photos for slideshows");
        ap.refer(&mut nb_levels)
            .add_option(&["-n","--nb_levels"], Store,"Number of levels to recurse during directory search (default 1)");
        ap.refer(&mut search_dir)
            .add_option(&["-d","--directory"], Store,"Name of directory holding the photos (default .)");
	ap.refer(&mut output_dir)
            .add_option(&["-o","--output"], Store, "Name of output directory (default /mnt/f/jpegs)");
	ap.refer(&mut tmp_dir)
            .add_option(&["-t","--tmp"], Store, "Temporary workspace (default /mnt/ramfs)");
	ap.refer(&mut cities)
            .add_option(&["-c","--cities"], Store, "File holding cities names (default ./cities.csv)");
	ap.refer(&mut exts)
            .add_option(&["-e","--exts"], Store, "string (not case sensitive) holding file extension(s) to process separated by commas (default jpg)");
        ap.parse_args_or_exit();
    }

    let vexts:Vec<String> = exts.split(',').map(|x| x.to_ascii_lowercase()).collect();
    let tab = read_cities(&cities);
    for entry in walkdir::WalkDir::new(search_dir)
	.max_depth(nb_levels)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        println!("Processing {}", entry.path().display());
        one(entry.path(), &tab, &vexts,&output_dir,&tmp_dir);
    }
}
