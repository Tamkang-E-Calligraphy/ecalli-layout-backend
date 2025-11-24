use ecalli_layout_backend::feature::json::{AnimateSubject, AnimationRequest};
use ecalli_layout_backend::feature::*;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let test_req = AnimationRequest {
        subject:"李白 登金陵鳳凰臺".into(),
        subject_font_type: "行書".into(),
        subject_list: Vec::new(),
        content:"鳳凰臺上鳳凰遊鳳去臺空江自流吳宮花草埋幽徑晉代衣冠成古丘三山半茖青天外二水中分白鷺洲總為浮雲能蔽日長安不見使人愁".into(),
        font_type:"行書".into(),
        word_list: vec![AnimateSubject{pos_x:631.,pos_y:43.,width:86,height:90},AnimateSubject{pos_x:631.,pos_y:153.,width:86,height:83},AnimateSubject{pos_x:631.,pos_y:256.,width:86,height:116},AnimateSubject{pos_x:631.,pos_y:392.,width:86,height:80},AnimateSubject{pos_x:631.,pos_y:492.,width:86,height:90},AnimateSubject{pos_x:631.,pos_y:602.,width:86,height:83},AnimateSubject{pos_x:631.,pos_y:705.,width:86,height:80},AnimateSubject{pos_x:631.,pos_y:805.,width:86,height:90},AnimateSubject{pos_x:631.,pos_y:915.,width:86,height:95},AnimateSubject{pos_x:535.,pos_y:43.,width:86,height:116},AnimateSubject{pos_x:535.,pos_y:179.,width:86,height:92},AnimateSubject{pos_x:535.,pos_y:291.,width:86,height:75},AnimateSubject{pos_x:535.,pos_y:386.,width:86,height:149},AnimateSubject{pos_x:535.,pos_y:555.,width:86,height:83},AnimateSubject{pos_x:535.,pos_y:658.,width:86,height:81},AnimateSubject{pos_x:535.,pos_y:759.,width:86,height:104},AnimateSubject{pos_x:535.,pos_y:883.,width:86,height:90},AnimateSubject{pos_x:439.,pos_y:43.,width:86,height:111},AnimateSubject{pos_x:439.,pos_y:174.,width:86,height:73},AnimateSubject{pos_x:439.,pos_y:267.,width:86,height:74},AnimateSubject{pos_x:439.,pos_y:361.,width:86,height:75},AnimateSubject{pos_x:439.,pos_y:456.,width:86,height:83},AnimateSubject{pos_x:439.,pos_y:559.,width:86,height:86},AnimateSubject{pos_x:439.,pos_y:665.,width:86,height:80},AnimateSubject{pos_x:439.,pos_y:765.,width:86,height:83},AnimateSubject{pos_x:439.,pos_y:868.,width:86,height:96},AnimateSubject{pos_x:343.,pos_y:43.,width:86,height:106},AnimateSubject{pos_x:343.,pos_y:169.,width:86,height:77},AnimateSubject{pos_x:343.,pos_y:266.,width:86,height:61},AnimateSubject{pos_x:343.,pos_y:347.,width:86,height:69},AnimateSubject{pos_x:343.,pos_y:436.,width:86,height:125},AnimateSubject{pos_x:343.,pos_y:581.,width:86,height:99},AnimateSubject{pos_x:343.,pos_y:700.,width:86,height:100},AnimateSubject{pos_x:343.,pos_y:820.,width:86,height:86},AnimateSubject{pos_x:343.,pos_y:926.,width:86,height:92},AnimateSubject{pos_x:247.,pos_y:43.,width:86,height:60},AnimateSubject{pos_x:247.,pos_y:123.,width:86,height:79},AnimateSubject{pos_x:247.,pos_y:222.,width:86,height:116},AnimateSubject{pos_x:247.,pos_y:358.,width:86,height:71},AnimateSubject{pos_x:247.,pos_y:449.,width:86,height:91},AnimateSubject{pos_x:247.,pos_y:560.,width:86,height:99},AnimateSubject{pos_x:247.,pos_y:679.,width:86,height:86},AnimateSubject{pos_x:247.,pos_y:785.,width:86,height:82},AnimateSubject{pos_x:247.,pos_y:887.,width:86,height:100},AnimateSubject{pos_x:151.,pos_y:43.,width:86,height:90},AnimateSubject{pos_x:151.,pos_y:153.,width:86,height:98},AnimateSubject{pos_x:151.,pos_y:271.,width:86,height:88},AnimateSubject{pos_x:151.,pos_y:379.,width:86,height:86},AnimateSubject{pos_x:151.,pos_y:485.,width:86,height:113},AnimateSubject{pos_x:151.,pos_y:618.,width:86,height:89},AnimateSubject{pos_x:151.,pos_y:727.,width:86,height:82},AnimateSubject{pos_x:151.,pos_y:829.,width:86,height:87},AnimateSubject{pos_x:151.,pos_y:936.,width:86,height:96},AnimateSubject{pos_x:55.,pos_y:43.,width:86,height:82},AnimateSubject{pos_x:55.,pos_y:145.,width:86,height:58},AnimateSubject{pos_x:55.,pos_y:223.,width:86,height:86}],
        width:760,
        height:1040
    };
    let webpdata = generate_poem_animation_webp(test_req, 33).await?;
    fs::write("test_output.webp", webpdata)?;

    Ok(())
}
