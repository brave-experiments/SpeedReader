use crate::predictor::predict;
use std::collections::HashMap;

pub const N_FEATURES: usize = 21;
pub const N_CLASSES: usize = 2;

pub struct Classifier {
    features_list: [f32; N_FEATURES],
}

impl Classifier {
    pub fn from_feature_map(features: HashMap<String, usize>) -> Classifier {
        //let features_list: [f32; N_FEATURES] = [0.0; N_FEATURES];

        let features_list = convert_map(features);
        Classifier { features_list }
    }

    pub fn classify(&self) -> usize {
        let result = predict(&self.features_list);
        result
    }
}

// helpers
fn convert_map(map: HashMap<String, usize>) -> [f32; N_FEATURES] {
    let mut slice: [f32; N_FEATURES] = [0.0; N_FEATURES];

    slice[0] = map["img"] as f32;
    slice[1] = map["a"] as f32;
    slice[2] = map["script"] as f32;
    slice[3] = map["text_blocks"] as f32;
    slice[4] = map["words"] as f32;
    slice[5] = map["blockquote"] as f32;
    slice[6] = map["dl"] as f32;
    slice[7] = map["div"] as f32;
    slice[8] = map["ol"] as f32;
    slice[9] = map["p"] as f32;
    slice[10] = map["pre"] as f32;
    slice[11] = map["table"] as f32;
    slice[12] = map["ul"] as f32;
    slice[13] = map["select"] as f32;
    slice[14] = map["article"] as f32;
    slice[15] = map["section"] as f32;
    slice[16] = map["url_depth"] as f32;
    slice[17] = map["amphtml"] as f32;
    slice[18] = map["fb_pages"] as f32;
    slice[19] = map["og_article"] as f32;
    slice[20] = map["schema_org"] as f32;
    //slice[21] = map["file_size"] as f32;

    slice
}
