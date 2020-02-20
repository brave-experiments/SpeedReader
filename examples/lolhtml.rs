extern crate reqwest;
extern crate speedreader;
extern crate url;

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use url::Url;

use speedreader::speedreader_streamer::AttributeRewrite;
use speedreader::speedreader_streamer::SiteConfiguration;
use speedreader::speedreader_streamer::SpeedReader;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let article_url = &args[1];

    let url = Url::parse(article_url).unwrap();

    let client = reqwest::blocking::Client::new();
    let data = client
        .get(article_url)
        .header("cookie", "__cfduid=df332bfea64fef76a0d4d40e16e69f6231582213861; __tdbbrowser=eb1a766c-3a86-41ce-a94f-95b7e1055c62; __tdbbrowser.sig=9pRaxvlsC0Qu7aWzPxIateXZwJY; kppid=uZwYqPZD685; kppid.sig=PPRprR_oValYHCSpl079FNUvSok; __pnahc=0; __tbc=%7Bjzx%7D6IEwEmVVpk2IztC48bSVYvDtm58drLFdNDflUBO8kNKKl_KeqYR0Y9r4l0B0Fl4IFPolt8vBxFV_0mKrgA4QolJEStVM3-z1pR0sSFmw5Z081jzxeDScXCiSEW5hBWI1NuocILhq4hXpagC26VdoOQ; __pat=-18000000; audience_segment=direct; audience_segment_source=direct; __pil=en_US; _pc_annoyed=1; __tdbsesh=eyJub3dJbk1pbnV0ZXMiOjI2MzcwMjM2LCJzZXNzaW9uSWQiOiI0MDljNjIyMi0zZDQ0LTRiMWItOWI0OS03ZTY5OWUwMDhhMTQiLCJkZnBCdWNrZXRJZCI6MTV9; __tdbsesh.sig=yb9Ol1irK3aOVbUEUgEIp913RZg; __tdb24=4; __adblocker=false; __pvi=%7B%22id%22%3A%22v-2020-02-20-15-51-02-890-oGHQ9bPsiK3cn8zK-01f65bc433e7e4cc0602bc8edf2f9159%22%2C%22domain%22%3A%22.thedailybeast.com%22%2C%22time%22%3A1582214285504%7D; xbc=%7Bjzx%7DG7vgpq-OQ1JwiQhmsq9uww8weCt3KHgE5JuaFzCgdJ-JA4hqhQzA4snm_A7Q4uZrm8Hck506iI0RZ0oRzJ7vTwBSALOH1IBan1VspdTBC-V7MljVlo9-NQ3uwJqsmdpvgvvDs2QOYrEadFahU2psyhn19Nht35iHWbhVT9m7A0bw5nXRPa5lu9vWv7mfgZf_qrwovugoHcSHivMzIQAqUofdB4cta_m5QaxE2MqngRDjXALWTEFpaRomx4lb_sJBCcgUPSr97EmVr-Pjvd8JfOE3RL0JFO659Atz9z5uDdUoImZJVmO3zTK9n7R4Fyb5lEBWo-g7arwljn2kK2ErrlE4na3YGECAxlBIohfRTd3RnK1mX-fxYsJqDNPI6wHvTuan3dNZ4qTqL4ZwiaWl_JjT83lJWDkmxtVrFY6NFyh_sC2k9U0lJawnp59VL-OsipeT6kgMPq5i47OoLrMH4dzQ1NNi2XhHpqHN6NO_MT_Csxl1rFDO5Cl7Cd3K3v54h-rdebtstEGrmBfnlvL8xe2J5fKHIok2Nx_nP2uupDmWv7K1irokc6IFGdIc0R1eUkNqMGtq0WLQqglfNcS9F8mvhfyKzMn9CmFrmt3gskbOfCBGZvjoJm_2gcRnOWroPyiSL_I-9mxahuohHaxve-vxYyrUlhao29h5RJy0vIbeWMwLjlYFE6hY7zrPErxgVt6XdO_xmsEM7Ljcyob2QkiBlQFQdYIaF6TgUvjzH3aneOjchMu3IwVMVGf26eNbeh7KO7He93emywFrh6iqQsSPeKe4CKjNxRKDuqy8y-V6VS2hx2k9uZQcCTs0fSEJcp3_pVKGTvxJ0EBt04oUpcFK4o9dO8X01_LpTEtAuv9Qal6LLcz9UAOzkUsRgStSkbHNG3TVqNOGNUqCDUINkNvbQ4AwbZqVHi8QiV8YWgVDpxZg5lh1GvigdmRg8Wp4boeDe9KJIoJGAo11h7L1bNHuVHR9w7wX7vTPDLEU713OutGd7eaphNdcMKU2-iv-; _pc_cc=pv3-50; OptanonAlertBoxClosed=2020-02-20T15:58:12.520Z; OptanonConsent=isIABGlobal=false&datestamp=Thu+Feb+20+2020+15%3A58%3A12+GMT%2B0000+(Greenwich+Mean+Time)&version=5.8.0&landingPath=NotLandingPage&groups=1%3A1%2C0_249383%3A1%2C2%3A1%2C0_249386%3A1%2C0_249385%3A1%2C0_249399%3A1%2C0_249388%3A1%2C0_249402%3A1%2C0_249389%3A1%2C0_249390%3A1%2C0_249393%3A1%2C0_249394%3A1%2C0_249395%3A1%2C0_249396%3A1%2C0_249397%3A1%2C0_249398%3A1%2C0_249400%3A1%2C0_249401%3A1%2C0_249403%3A1%2C0_249409%3A1%2C0_249525%3A1")
        .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.106 Safari/537.36")
        .send()
        .unwrap()
        .text()
        .unwrap();

    let dir = format!(
        "data/lolhtml/{}/{}",
        url.host().unwrap(),
        calculate_hash(article_url)
    );
    println!("Creating directory: {}", dir);
    fs::create_dir_all(&dir).unwrap_or_default();

    let filename_html = format!("{}/init.html", &dir);
    let mut file = fs::File::create(filename_html).unwrap();
    file.write_all(data.as_bytes()).unwrap();

    let filename_html = format!("{}/init.html", "data/lolhtml/dump/test");
    let mut file = fs::File::create(filename_html).unwrap();
    file.write_all(data.as_bytes()).unwrap();

    let mut output = vec![];
    let sr = SpeedReader::configure(
        &SiteConfiguration {
            domain: "businessinsider.com".to_owned(),
            main_content: vec![
                ".post-headline", ".byline-wrapper",
                "#l-content", ".container figure",
            ],
            main_content_cleanup: vec![
                ".share-wrapper", ".ad",
                ".category-tagline",
                ".popular-video",
                "figure .lazy-image", "figure .lazy-blur",
            ],
            delazify: true,
            fix_embeds: false,
            content_script: None,
            preprocess: vec![
                AttributeRewrite {
                    selector: "figure noscript".to_owned(),
                    attribute: "id".to_owned(),
                    to_attribute: "id".to_owned(),
                    element_name: "div".to_owned()
                }
            ],
        },
        &url.origin().ascii_serialization(),
    );
    sr.rewrite(data.as_bytes(), &mut output)?;

    let filename_html = format!("{}/mapped.html", &dir);
    let mut file = fs::File::create(filename_html).unwrap();
    file.write_all(&output).unwrap();

    let filename_html = format!("{}/mapped.html", "data/lolhtml/dump/test");
    let mut file = fs::File::create(filename_html).unwrap();
    file.write_all(&output).unwrap();
    Ok(())
}
