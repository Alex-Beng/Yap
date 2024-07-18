use std::fs;

use tract_onnx::prelude::*;
use serde_json::Value;

use crate::capture::RawImage;

type ModelType = RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct CRNNModel {
    model: ModelType,
    model5: ModelType,
    index_2_word: Vec<String>,

    pub avg_inference_time: f64,
}

impl CRNNModel {
    // 我测，真就不用名字，直接硬编码啊
    // update: 这里硬编码无所谓，是编译期进去的
    pub fn new(_name: String, _dict_name: String) -> CRNNModel {
        /* 
        let bytes = include_bytes!("../../models/model_training.onnx");
        
        let model = tract_onnx::onnx()
            .model_for_read(&mut bytes.as_bytes()).unwrap()
            .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 1, 32, 384))).unwrap()
            .into_optimized().unwrap()
            .into_runnable().unwrap();
        
        // 一次推理五张图
        let model5 = tract_onnx::onnx()
        .model_for_read(&mut bytes.as_bytes()).unwrap()
        .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec!(5, 1, 32, 384))).unwrap()
        .into_optimized().unwrap()
        .into_runnable().unwrap();
        

        // let content = utils::read_file_to_string(String::from("models/index_2_word.json"));
        let content = String::from(include_str!("../../models/index_2_word.json"));
        let json: Value = serde_json::from_str(content.as_str()).unwrap();
        */

        let model = tract_onnx::onnx()
            .model_for_path("./model_training.onnx").unwrap()
            .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec!(1, 1, 32, 384))).unwrap()
            .into_optimized().unwrap()
            .into_runnable().unwrap();
        let model5 = tract_onnx::onnx()
            .model_for_path("./model_training.onnx").unwrap()
            .with_input_fact(0, InferenceFact::dt_shape(f32::datum_type(), tvec!(5, 1, 32, 384))).unwrap()
            .into_optimized().unwrap()
            .into_runnable().unwrap();
        let content = fs::read_to_string("./index_2_word.json").unwrap();
        let json: Value = serde_json::from_str(content.as_str()).unwrap();



        let mut index_2_word: Vec<String> = Vec::new();
        let mut i = 0;
        while let Some(word) = json.get(i.to_string()) {
            index_2_word.push(word.as_str().unwrap().to_string());
            i += 1;
        }

        CRNNModel {
            model,
            model5,
            index_2_word,

            avg_inference_time: 0.0,
        }
    }

    pub fn inference_string(&self, img: &RawImage) -> String {
        let tensor: Tensor = tract_ndarray::Array4::from_shape_fn((1, 1, 32, 384), |(_, _, y, x)| {
            let index = img.w * y as u32 + x as u32;
            img.data[index as usize]
        }).into();

        let result = self.model.run(tvec!(tensor.into())).unwrap();
        let arr = result[0].to_array_view::<f32>().unwrap();

        let shape = arr.shape();

        let mut ans = String::new();
        let mut last_word = String::new();
        for i in 0..shape[0] {
            let mut max_index = 0;
            let mut max_value = -1.0;
            for j in 0..self.index_2_word.len() {
                let value = arr[[i, 0, j]];
                if value > max_value {
                    max_value = value;
                    max_index = j;
                }
            }
            let word = &self.index_2_word[max_index];
            if *word != last_word && word != "|" {
                ans = ans + word;
            }
            last_word.clone_from(word);
        }

        ans
    }
    pub fn inference_5strings(&self, imgs: [&RawImage; 5]) -> [String; 5] {
        // 似乎用不太上，跟直接用五次 1, 1, 32, 384的推理时间差不多
        let mut strings:[String; 5] = [
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
            String::from(""),
        ];

        let tensor: Tensor = tract_ndarray::Array4::from_shape_fn((5, 1, 32, 384), |(i, _, y, x)| {
            let index = imgs[i].w * y as u32 + x as u32;
            imgs[i].data[index as usize]
        }).into();

        let result = self.model5.run(tvec!(tensor.into())).unwrap();
        let arr = result[0].to_array_view::<f32>().unwrap();

        let shape = arr.shape();

        // info!("arr shape: {:?}", arr.shape());
        for string in &mut strings {
            
            let mut ans = String::new();
            let mut last_word = String::new();
            for j in 0..shape[0] {
                let mut max_index = 0;
                let mut max_value = -1.0;
                for k in 0..self.index_2_word.len() {
                    let value = arr[[j, 0, k]];
                    if value > max_value {
                        max_value = value;
                        max_index = k;
                    }
                }
                let word = &self.index_2_word[max_index];
                if *word != last_word && word != "|" {
                    ans = ans + word;
                }

                last_word.clone_from(word);
            }
            *string = ans;
        }
        
        
        strings
    }
}