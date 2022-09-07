use crate::NumberOfFields;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Color { 
    pub r: f32,
    pub g: f32,
    pub b: f32
}

impl NumberOfFields for Color {
    fn nr_fields(&self) -> usize {
        return 3;
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(rgb: (u8, u8, u8)) -> Color {
        Color {
            r: rgb.0 as f32 / 255.0,
            g: rgb.1 as f32 / 255.0,
            b: rgb.2 as f32 / 255.0,
        }
    }
}

impl FromStr for Color {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
			"black" => Ok((0,0,0).into()),
			"white" => Ok((255,255,255).into()),
			"red" => Ok((255,0,0).into()),
			"lime" => Ok((0,255,0).into()),
			"blue" => Ok((0,0,255).into()),
			"yellow" => Ok((255,255,0).into()),
			"cyan" => Ok((0,255,255).into()),
			"magenta" => Ok((255,0,255).into()),
			"silver" => Ok((192,192,192).into()),
			"gray" => Ok((128,128,128).into()),
			"maroon" => Ok((128,0,0).into()),
			"olive" => Ok((128,128,0).into()),
			"green" => Ok((0,128,0).into()),
			"purple" => Ok((128,0,128).into()),
			"teal" => Ok((0,128,128).into()),
			"navy" => Ok((0,0,128).into()),
			"dark red" => Ok((139,0,0).into()),
			"brown" => Ok((165,42,42).into()),
			"firebrick" => Ok((178,34,34).into()),
			"crimson" => Ok((220,20,60).into()),
			"tomato" => Ok((255,99,71).into()),
			"coral" => Ok((255,127,80).into()),
			"indian red" => Ok((205,92,92).into()),
			"light coral" => Ok((240,128,128).into()),
			"dark salmon" => Ok((233,150,122).into()),
			"salmon" => Ok((250,128,114).into()),
			"light salmon" => Ok((255,160,122).into()),
			"orange red" => Ok((255,69,0).into()),
			"dark orange" => Ok((255,140,0).into()),
			"orange" => Ok((255,165,0).into()),
			"gold" => Ok((255,215,0).into()),
			"dark golden rod" => Ok((184,134,11).into()),
			"golden rod" => Ok((218,165,32).into()),
			"pale golden rod" => Ok((238,232,170).into()),
			"dark khaki" => Ok((189,183,107).into()),
			"khaki" => Ok((240,230,140).into()),
			"yellow green" => Ok((154,205,50).into()),
			"dark olive green" => Ok((85,107,47).into()),
			"olive drab" => Ok((107,142,35).into()),
			"lawn green" => Ok((124,252,0).into()),
			"chartreuse" => Ok((127,255,0).into()),
			"green yellow" => Ok((173,255,47).into()),
			"dark green" => Ok((0,100,0).into()),
			"forest green" => Ok((34,139,34).into()),
			"lime green" => Ok((50,205,50).into()),
			"light green" => Ok((144,238,144).into()),
			"pale green" => Ok((152,251,152).into()),
			"dark sea green" => Ok((143,188,143).into()),
			"medium spring green" => Ok((0,250,154).into()),
			"spring green" => Ok((0,255,127).into()),
			"sea green" => Ok((46,139,87).into()),
			"medium aqua marine" => Ok((102,205,170).into()),
			"medium sea green" => Ok((60,179,113).into()),
			"light sea green" => Ok((32,178,170).into()),
			"dark slate gray" => Ok((47,79,79).into()),
			"dark cyan" => Ok((0,139,139).into()),
			"aqua" => Ok((0,255,255).into()),
			"light cyan" => Ok((224,255,255).into()),
			"dark turquoise" => Ok((0,206,209).into()),
			"turquoise" => Ok((64,224,208).into()),
			"medium turquoise" => Ok((72,209,204).into()),
			"pale turquoise" => Ok((175,238,238).into()),
			"aqua marine" => Ok((127,255,212).into()),
			"powder blue" => Ok((176,224,230).into()),
			"cadet blue" => Ok((95,158,160).into()),
			"steel blue" => Ok((70,130,180).into()),
			"corn flower blue" => Ok((100,149,237).into()),
			"deep sky blue" => Ok((0,191,255).into()),
			"dodger blue" => Ok((30,144,255).into()),
			"light blue" => Ok((173,216,230).into()),
			"sky blue" => Ok((135,206,235).into()),
			"light sky blue" => Ok((135,206,250).into()),
			"midnight blue" => Ok((25,25,112).into()),
			"dark blue" => Ok((0,0,139).into()),
			"medium blue" => Ok((0,0,205).into()),
			"royal blue" => Ok((65,105,225).into()),
			"blue violet" => Ok((138,43,226).into()),
			"indigo" => Ok((75,0,130).into()),
			"dark slate blue" => Ok((72,61,139).into()),
			"slate blue" => Ok((106,90,205).into()),
			"medium slate blue" => Ok((123,104,238).into()),
			"medium purple" => Ok((147,112,219).into()),
			"dark magenta" => Ok((139,0,139).into()),
			"dark violet" => Ok((148,0,211).into()),
			"dark orchid" => Ok((153,50,204).into()),
			"medium orchid" => Ok((186,85,211).into()),
			"thistle" => Ok((216,191,216).into()),
			"plum" => Ok((221,160,221).into()),
			"violet" => Ok((238,130,238).into()),
			"orchid" => Ok((218,112,214).into()),
			"medium violet red" => Ok((199,21,133).into()),
			"pale violet red" => Ok((219,112,147).into()),
			"deep pink" => Ok((255,20,147).into()),
			"hot pink" => Ok((255,105,180).into()),
			"light pink" => Ok((255,182,193).into()),
			"pink" => Ok((255,192,203).into()),
			"antique white" => Ok((250,235,215).into()),
			"beige" => Ok((245,245,220).into()),
			"bisque" => Ok((255,228,196).into()),
			"blanched almond" => Ok((255,235,205).into()),
			"wheat" => Ok((245,222,179).into()),
			"corn silk" => Ok((255,248,220).into()),
			"lemon chiffon" => Ok((255,250,205).into()),
			"light golden rod yellow" => Ok((250,250,210).into()),
			"light yellow" => Ok((255,255,224).into()),
			"saddle brown" => Ok((139,69,19).into()),
			"sienna" => Ok((160,82,45).into()),
			"chocolate" => Ok((210,105,30).into()),
			"peru" => Ok((205,133,63).into()),
			"sandy brown" => Ok((244,164,96).into()),
			"burly wood" => Ok((222,184,135).into()),
			"tan" => Ok((210,180,140).into()),
			"rosy brown" => Ok((188,143,143).into()),
			"moccasin" => Ok((255,228,181).into()),
			"navajo white" => Ok((255,222,173).into()),
			"peach puff" => Ok((255,218,185).into()),
			"misty rose" => Ok((255,228,225).into()),
			"lavender blush" => Ok((255,240,245).into()),
			"linen" => Ok((250,240,230).into()),
			"old lace" => Ok((253,245,230).into()),
			"papaya whip" => Ok((255,239,213).into()),
			"sea shell" => Ok((255,245,238).into()),
			"mint cream" => Ok((245,255,250).into()),
			"slate gray" => Ok((112,128,144).into()),
			"light slate gray" => Ok((119,136,153).into()),
			"light steel blue" => Ok((176,196,222).into()),
			"lavender" => Ok((230,230,250).into()),
			"floral white" => Ok((255,250,240).into()),
			"alice blue" => Ok((240,248,255).into()),
			"ghost white" => Ok((248,248,255).into()),
			"honeydew" => Ok((240,255,240).into()),
			"ivory" => Ok((255,255,240).into()),
			"azure" => Ok((240,255,255).into()),
			"snow" => Ok((255,250,250).into()),
			"dim gray" => Ok((105,105,105).into()),
			"dark gray" => Ok((169,169,169).into()),
			"light gray" => Ok((211,211,211).into()),
			"gainsboro" => Ok((220,220,220).into()),
			"white smoke" => Ok((245,245,245).into()),
            s => Err(format!("invalid color \"{}\"", s)),
        }
    }
}

impl Color {
    pub fn iter(&self) -> std::vec::IntoIter<f32> {
        vec!(
            self.r,
            self.g,
            self.b,
        ).into_iter()
    }
}

#[macro_export]
macro_rules! col {
    () => {
        col!(0.827, 0.827, 0.827)
    };
    (black) => {
        col!(0.0,0.0,0.0)
    };
    (red) => {
        col!(1.0,0.0,0.0)
    };
    (green) => {
        col!(0.0,1.0,0.0)
    };
    (blue) => {
        col!(0.0,0.0,1.0)
    };
    (white) => {
        col!(1.0,1.0,1.0)
    };
    ($r:expr, $g:expr, $b:expr) => {
        Color {
            r: $r,
            g: $g,
            b: $b,
        }
    };
}
